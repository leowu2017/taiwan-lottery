use std::time::Duration;

use reqwest::blocking::{Client, RequestBuilder, Response};

use crate::DownloadError;

const USER_AGENT: &str = concat!("taiwan-lottery/", env!("CARGO_PKG_VERSION"));

/// Pause between consecutive requests to the same upstream service.
#[cfg(not(test))]
const REQUEST_PAUSE: Duration = Duration::from_millis(200);

#[derive(Debug, Clone, Copy)]
pub(crate) struct RetryPolicy {
    pub(crate) max_attempts: u32,
    pub(crate) base_delay: Duration,
}

pub(crate) const DEFAULT_RETRY: RetryPolicy = RetryPolicy {
    max_attempts: 3,
    base_delay: Duration::from_millis(500),
};

pub(crate) fn build_http_client() -> Result<Client, DownloadError> {
    Client::builder()
        .timeout(Duration::from_secs(60))
        .user_agent(USER_AGENT)
        .build()
        .map_err(DownloadError::from)
}

/// Gives the upstream services a short breather between sequential requests.
pub(crate) fn polite_pause() {
    #[cfg(not(test))]
    std::thread::sleep(REQUEST_PAUSE);
}

// HTTP 500 is not retried: the Taiwan Lottery API uses it to signal "no such resource".
fn is_retryable_status(status: reqwest::StatusCode) -> bool {
    matches!(status.as_u16(), 429 | 502 | 503 | 504)
}

fn is_retryable_error(err: &reqwest::Error) -> bool {
    err.is_timeout() || err.is_connect()
}

/// Sends a request, retrying transient failures with exponential backoff, and returns the
/// response whatever its status.
pub(crate) fn send_raw_with_policy(
    policy: RetryPolicy,
    build: impl Fn() -> RequestBuilder,
) -> Result<Response, DownloadError> {
    let mut attempt = 1u32;
    loop {
        match build().send() {
            Ok(response)
                if is_retryable_status(response.status()) && attempt < policy.max_attempts => {}
            Ok(response) => return Ok(response),
            Err(err) if is_retryable_error(&err) && attempt < policy.max_attempts => {}
            Err(err) => return Err(err.into()),
        }

        std::thread::sleep(policy.base_delay * 2u32.pow(attempt - 1));
        attempt += 1;
    }
}

pub(crate) fn send_raw(build: impl Fn() -> RequestBuilder) -> Result<Response, DownloadError> {
    send_raw_with_policy(DEFAULT_RETRY, build)
}

/// Like [`send_raw`], but turns non-success statuses into errors.
pub(crate) fn send_ok(build: impl Fn() -> RequestBuilder) -> Result<Response, DownloadError> {
    send_raw(build)?.error_for_status().map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    use super::*;
    use crate::test_support::{MockResponse, MockServer};

    const FAST: RetryPolicy = RetryPolicy {
        max_attempts: 3,
        base_delay: Duration::from_millis(1),
    };

    #[test]
    fn transient_status_is_retried_until_success() {
        let calls = Arc::new(AtomicUsize::new(0));
        let handler_calls = Arc::clone(&calls);
        let server = MockServer::start(move |_| {
            if handler_calls.fetch_add(1, Ordering::SeqCst) < 2 {
                MockResponse::status(503)
            } else {
                MockResponse::ok("done")
            }
        });
        let client = build_http_client().expect("client");

        let response = send_raw_with_policy(FAST, || client.get(&server.base_url)).expect("send");
        assert!(response.status().is_success());
        assert_eq!(server.hits(), 3);
    }

    #[test]
    fn retries_stop_after_the_attempt_limit() {
        let server = MockServer::start(|_| MockResponse::status(503));
        let client = build_http_client().expect("client");

        let response = send_raw_with_policy(FAST, || client.get(&server.base_url)).expect("send");
        assert_eq!(response.status().as_u16(), 503);
        assert_eq!(server.hits(), 3);
    }

    #[test]
    fn internal_server_error_is_not_retried() {
        let server = MockServer::start(|_| MockResponse::status(500));
        let client = build_http_client().expect("client");

        let response = send_raw_with_policy(FAST, || client.get(&server.base_url)).expect("send");
        assert_eq!(response.status().as_u16(), 500);
        assert_eq!(server.hits(), 1);
    }

    #[test]
    fn requests_carry_the_crate_user_agent() {
        let server = MockServer::start(|_| MockResponse::ok("ok"));
        let client = build_http_client().expect("client");

        let response = client.get(&server.base_url).send().expect("send");
        assert!(response.status().is_success());
        let request = server.last_request().to_lowercase();
        assert!(request.contains(&format!("user-agent: {}", USER_AGENT.to_lowercase())));
    }
}
