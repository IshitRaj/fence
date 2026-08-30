# Rust Standard Library

Fence currently uses the Rust standard library wherever possible.

## Used so far

* `std::path::{Path, PathBuf}` — filesystem path handling.
* `std::env` — environment variables such as `HOME`.
* `Vec<T>` — storing policy rules.
* `Result<T, E>` — parser error handling.
* `str` methods and iterators — parsing `.fence` input without a parsing dependency.

## External dependencies

None currently required.

## When stdlib isn't enough

If Fence needs functionality that Rust's standard library genuinely cannot provide, the reason and chosen solution will be documented here.
