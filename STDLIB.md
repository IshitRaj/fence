# Rust Standard Library

Fence currently uses the Rust standard library wherever possible.

## Used so far

* `std::path::{Path, PathBuf}` — filesystem path handling.
* `std::fs` — reading, writing, and removing files (`read`, `write`, `read_to_string`, `remove_file`) for both the library's `read`/`write` operations and test fixtures.
* `std::io::Error` — underlying I/O failures, wrapped in `FenceOperationError::Io` / `FenceError::Io`.
* `std::env` — environment variables such as `HOME`, and `temp_dir()` for locating the platform's temp directory (used in tests instead of hardcoding `/tmp`, since that path differs on macOS).
* `Vec<T>` — storing policy rules.
* `Result<T, E>` — parser and policy-check error handling.
* `str` methods and iterators — parsing `.fence` input without a parsing dependency.

## Test-only usage

Not part of the library's runtime behavior, used only to keep the test suite isolated when run in parallel:

* `std::sync::atomic::{AtomicU64, Ordering}` — per-process counter for generating unique temp file names, avoiding path collisions between tests.
* `std::time::{SystemTime, UNIX_EPOCH}` — nanosecond component of unique temp file names.
* `std::process::id()` — process-id component of unique temp file names, in case multiple test binaries run concurrently.

## External dependencies

None currently required.

## When stdlib isn't enough

If Fence needs functionality that Rust's standard library genuinely cannot provide, the reason and chosen solution will be documented here.