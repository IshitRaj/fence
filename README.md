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

## Ask rules

A rule marked `ask` is not automatically allowed or denied. It needs a decision at runtime, made by an approval handler.

```rust
let fence = Fence::load(".fence")?.with_approval_handler(|request| {
    // show the request to a human, a log, a prompt, whatever fits
    // return ApprovalDecision::Approved or ApprovalDecision::Denied
});
```

If no handler is configured, an operation that hits an `ask` rule returns `FenceOperationError::Ask`, carrying the request that needed a decision. This fails loudly rather than silently allowing or denying, so any policy using `ask` needs a handler configured.

## Playground

A small runnable example is provided in `examples/` to experiment with a real `.fence` policy and the Fence API.

Run it with:

```bash
cargo run --example playground
```

The playground demonstrates reading, writing, and deleting a file, prompting for approval on anything marked `ask`. The test file is deleted at the end, so it must be created again before another run.

## Development

```bash
cargo check
cargo test
```

Fence is currently under development and is not yet a complete sandbox.