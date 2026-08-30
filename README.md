# Fence

Fence is a policy-based security layer for controlling filesystem, process, and network access.

Policies are written in `.fence` files:

```toml
[filesystem]
allow read ./projects/**
allow write ./playground/**
deny delete ./playground/**

[process]
allow command cargo, rustc
allow scope ./playground/**

[network]
allow host api.github.com
deny host *
```

## Status

* Filesystem — Available
* Process — In development
* Network — In development

Filesystem currently supports `read`, `write`, and `delete` through the `Fence` API.

## Playground

A small runnable example is available in `examples/` for experimenting with a real `.fence` policy.

```bash
cargo run --example playground
```

It demonstrates reading, writing, and deleting a file using relative paths.

The test file is deleted at the end of the run, so create it again before running the example again.

## Development

```bash
cargo check
cargo test
```

Fence is currently in development and is not yet a complete sandbox.
