# CLAUDE.md

Development guidelines for `taiwan-lottery`. Keep this file high-level; read the code for details, since it changes often.

## Project Nature

- A library that downloads, queries, and randomly draws Taiwan lottery data.
- Rust is the primary implementation. The C API is a thin FFI layer (`cdylib` + `staticlib`) over the Rust code, with C examples and tests under `c/`.
- Data comes from external sources (government open data and the Taiwan Lottery web API), so network, parsing, and data-shape failures are expected and must be handled gracefully.

## Language and Communication

- Everything committed to the repo must be in English: code, identifiers, comments, docs, commit messages, error messages, and PR descriptions.
- Chinese is allowed only where it is data or user-facing content (e.g., Chinese display names of games, test fixtures from Chinese sources).
- Keep comments rare and short; explain only what the code cannot show (the "why").

## Architecture Principles

- **Rust is the single source of truth.** Business logic, game rules, and data handling live in Rust. The C layer must not reimplement logic.
- **Keep FFI thin.** The FFI layer only converts arguments, maps errors to status codes, and manages memory. No domain logic in FFI or in C code.
- **Rust and C APIs must stay aligned.** Any public API, enum, constant, or struct change in Rust needs the matching C header, FFI, docs, examples, and tests updated in the same change. Rust and C enum/integer mappings must stay in sync.
- **Separate concerns by module:** download, query (local vs. remote), draw, number models, game rules/metadata, errors, FFI. Do not mix I/O with pure logic; keep pure logic easy to test.
- **Game-specific rules are data-driven** where possible. Adding a game should mean extending the central rule/metadata definitions, not scattering special cases.
- **Public API stability matters.** Prefer additive changes; deprecate with `#[deprecated]` and a migration note instead of removing in a minor or patch release. Follow semver. Remove deprecated APIs (Rust, FFI, C headers, docs, examples, tests) in the next major version bump whenever possible.
- Do not add dependencies lightly. Justify each new crate and keep the TLS stack and feature flags minimal.

## Rust Guidelines

- Stable toolchain only; code must pass `cargo fmt` and `cargo clippy` without new warnings.
- Return `Result` with the crate's typed errors. No `unwrap`/`expect`/`panic!` in library code paths (tests and examples excepted).
- Panics must never cross the FFI boundary.
- Prefer strong types (enums, newtypes) over stringly-typed values at API boundaries; parse and validate input at the edge.
- Document public items with concise rustdoc.

## C / FFI Guidelines

- Every `extern "C"` function must validate pointers and UTF-8 input and return a defined status code; never assume valid input.
- Memory ownership must be explicit: anything allocated by Rust is freed by a matching `free_*` function exported by Rust, never by the C `free`.
- `#[repr(C)]` layouts and header declarations must match exactly. Treat layout changes as breaking.
- C code is standard C, warning-clean, and portable across Windows, Linux, and macOS (the build is CMake-based).
- Headers are split by concern under `include/taiwan_lottery/`; keep the umbrella header in sync.

## Data and Network

- The `data/` directory is downloaded output and is git-ignored. Never commit downloaded datasets or build artifacts (`target/`, `c/build/`).
- Tests must not depend on live network access unless explicitly marked as such; prefer local fixtures or the downloaded data with clear skip behavior.
- Be respectful to upstream services: no aggressive polling, and handle HTTP errors and format changes without crashing.

## Testing

- New behavior requires tests. Bug fixes require a regression test.
- Rust tests: unit tests next to the code, integration tests in `tests/`. A parity test guards Rust vs. remote/local consistency.
- C tests under `c/tests/` must cover FFI-visible behavior, including memory release and error codes.
- When touching either interface, run both the Rust and C test suites (build the Rust release library first, then CMake/CTest for C).

## Review Before Commit

Before every commit, review your own diff (`git diff`) and confirm:

1. The change is minimal, focused, and does only what was asked; no unrelated refactors or reformatting.
2. `cargo fmt`, `cargo clippy`, and `cargo test` pass; for any API/FFI-touching change, the C build and CTest also pass.
3. Rust API, FFI, C headers, C examples, and docs (README files) are consistent with each other.
4. No secrets, local paths, downloaded data, or build output are included.
5. No leftover debug code, dead code, commented-out code, or TODOs without context.
6. Everything is in English and follows the guidelines above.

## Commits

- Use Conventional Commits: `<type>(<optional scope>): <imperative English summary>` (e.g., `feat(query): add Lotto 3D query range`). Types: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`. Scope is a module name such as `query`, `download`, `draw`, `ffi`, or `c`.
- Before committing, check recent `git log` and match its style; keep the format consistent with existing history.
- Small, atomic commits; keep the subject concise with no trailing period.
- Explain the "why" in the body when it is not obvious.
- Do not rewrite existing history to normalize old messages without explicit user approval.
- Never push or perform any action that affects a remote or shared system (force-push, rewriting published history, publishing to crates.io, creating or modifying tags, releases, branches, issues, or PRs on the remote) without explicit user approval for that specific action. Local commits are fine; anything remote requires asking first.

## Working With Agents

- Read the relevant code before changing it; do not rely on this file for implementation details.
- Prefer editing existing files over creating new ones; do not add documentation files unless asked.
- Update README content when public behavior, commands, or APIs change.
