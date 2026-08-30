# Rust Standard Library

Fence currently uses the Rust standard library wherever possible.

## Used so far

* `std::path::{Path, PathBuf}` — filesystem path handling.
* `std::fs` — reading, writing, removing, and checking files (`read`, `write`, `read_to_string`, `remove_file`, `exists`) for the library's `read`/`write`/`delete` operations and test fixtures.
* `std::process::{Command, Output}` — spawning and capturing the result of scoped commands in `execute`.
* `std::net::TcpStream` — opening outbound connections in `connect`.
* `std::io::Error` — underlying I/O failures, wrapped in `FenceOperationError::Io` / `FenceError::Io`.
* `std::fmt::{Display, Formatter}` — human-readable messages for `FenceOperationError` and `FenceRequest`, so a denied or pending request prints something actionable instead of a raw Debug dump.
* `std::error::Error` — implemented for `FenceOperationError`, so it composes with `?` and `Box<dyn Error>`, and exposes the wrapped `io::Error` through `source()`.
* `std::sync::Arc` — shared ownership of the optional `dyn ApprovalHandler` on `Fence`, so every operation call can consult the same handler without owning it.
* `std::env` — environment variables such as `HOME`, and `temp_dir()` for locating the platform's temp directory (used in tests instead of hardcoding `/tmp`, since that path differs on macOS).
* `Vec<T>` — storing policy rules.
* `Result<T, E>` — parser, policy-check, and operation error handling.
* `str` methods and iterators — parsing `.fence` input without a parsing dependency.

## Example-only usage

Used in `examples/playground.rs`, not part of the library itself:

* `std::io::{self, Write}` — reading a y/n answer from stdin and flushing the prompt to stdout for the approval handler.

## Test-only usage

Not part of the library's runtime behavior, used only to keep the test suite isolated when run in parallel:

* `std::sync::atomic::{AtomicU64, Ordering}` — per-process counter for generating unique temp file names, avoiding path collisions between tests.
* `std::time::{SystemTime, UNIX_EPOCH}` — nanosecond component of unique temp file names.
* `std::process::id()` — process-id component of unique temp file names, in case multiple test binaries run concurrently.

## External dependencies

None currently required.

## When stdlib isn't enough

If Fence needs functionality that Rust's standard library genuinely cannot provide, the reason and chosen solution will be documented here.