# Fence

Fence is a policy-based security layer for controlling filesystem, process, and network access.

Policies are written in `.fence` files.

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

Fence supports:

* Filesystem: `read`, `write`, `delete`
* Process: `command`, `scope`
* Network: `host`
* `allow`, `ask`, and `deny` rules
* Relative, absolute, and `~` paths
* `*` and `**` path patterns

## Playground

A small runnable example is provided in `examples/` to experiment with a real `.fence` policy and the Fence API.

Run it with:

```bash
cargo run --example playground
```

The playground demonstrates reading, writing, and deleting a file. The test file is deleted at the end, so it must be created again before another run.

## Development

```bash
cargo check
cargo test
```

Fence is currently under development and is not yet a complete sandbox.
