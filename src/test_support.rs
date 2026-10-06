use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

pub(crate) struct MockResponse {
    pub(crate) status: u16,
    pub(crate) body: Vec<u8>,
}

impl MockResponse {
    pub(crate) fn ok(body: impl Into<Vec<u8>>) -> Self {
        Self {
            status: 200,
            body: body.into(),
        }
    }

    pub(crate) fn status(status: u16) -> Self {
        Self {
            status,
            body: Vec::new(),
        }
    }
}

/// Minimal local HTTP server so network code can be tested without live access.
pub(crate) struct MockServer {
    pub(crate) base_url: String,
    hits: Arc<AtomicUsize>,
    last_request: Arc<Mutex<String>>,
}

impl MockServer {
    /// Starts a server that answers every request with `handler(request_target)`.
    pub(crate) fn start<F>(handler: F) -> Self
    where
        F: Fn(&str) -> MockResponse + Send + 'static,
    {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock server");
        let base_url = format!("http://{}", listener.local_addr().expect("local addr"));
        let hits = Arc::new(AtomicUsize::new(0));
        let thread_hits = Arc::clone(&hits);
        let last_request = Arc::new(Mutex::new(String::new()));
        let thread_last_request = Arc::clone(&last_request);

        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let mut request = Vec::new();
                let mut chunk = [0u8; 1024];
                while !request.windows(4).any(|window| window == b"\r\n\r\n") {
                    match stream.read(&mut chunk) {
                        Ok(0) | Err(_) => break,
                        Ok(read) => request.extend_from_slice(&chunk[..read]),
                    }
                }

                let text = String::from_utf8_lossy(&request);
                let target = text
                    .lines()
                    .next()
                    .and_then(|line| line.split_whitespace().nth(1))
                    .unwrap_or("/")
                    .to_string();

                thread_hits.fetch_add(1, Ordering::SeqCst);
                if let Ok(mut last) = thread_last_request.lock() {
                    *last = text.to_string();
                }
                let response = handler(&target);
                let head = format!(
                    "HTTP/1.1 {} Mock\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    response.status,
                    response.body.len()
                );
                let _ = stream.write_all(head.as_bytes());
                let _ = stream.write_all(&response.body);
            }
        });

        Self {
            base_url,
            hits,
            last_request,
        }
    }

    pub(crate) fn hits(&self) -> usize {
        self.hits.load(Ordering::SeqCst)
    }

    /// Raw text of the most recent request, including headers.
    pub(crate) fn last_request(&self) -> String {
        self.last_request
            .lock()
            .map(|text| text.clone())
            .unwrap_or_default()
    }
}

pub(crate) fn zip_bytes(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for (name, content) in entries {
        writer
            .start_file(*name, zip::write::SimpleFileOptions::default())
            .expect("start zip entry");
        writer.write_all(content).expect("write zip entry");
    }
    writer.finish().expect("finish zip").into_inner()
}
