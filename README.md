# Fence

A zero-dependency policy engine for filesystem, process, and network access in Rust.

## Status

v0.0.1. Built as a 72-hour hackathon project. The allow/ask/deny model, the `.fence` file format, and the approval flow below are implemented and tested. Anything not mentioned in this README isn't built yet.

## What Fence does

Fence sits between your application code and `std::fs`, `std::process`, and `std::net`. Every filesystem read, write, or delete, every spawned command, and every outbound connection you route through the `Fence` API is checked against a policy file before it runs. The policy decides whether the operation is allowed outright, denied outright, or requires an explicit approval at runtime.

Fence has no external dependencies. The `.fence` file parser and everything else is built on the Rust standard library alone.

## Installation

Not yet published to crates.io. Point Cargo at a path or git dependency until it is:

```toml
[dependencies]
fence = { path = "../fence" }
```

## Quick start

```rust
use fence::Fence;

fn main() {
    let fence = Fence::load(".fence").expect("failed to load policy");

    fence.write("output/report.txt", b"hello").expect("write failed");
    let content = fence.read("output/report.txt").expect("read failed");

    println!("{}", String::from_utf8_lossy(&content));
}
```

## Policy files (`.fence`)

```toml
[filesystem]
allow read ./projects/**
allow write ./playground/**
ask delete ./playground/**

[process]
allow command cargo, rustc
ask command rm
allow scope ./playground/**

[network]
allow host api.github.com
ask host *.internal.example.com
deny host *
```

Every rule falls into `allow`, `ask`, or `deny`. If a request doesn't match any rule at all, it's denied by default, nothing is implicitly allowed. If more than one rule could match, Fence checks in this order: `deny` first, then `ask`, then `allow`. The most restrictive match always wins.

**Filesystem** rules are split into `read`, `write`, and `delete`, each with its own independent list.

**Process** rules gate on the command name (`allow command cargo, rustc`) and separately require a `scope`, a path glob the working directory must fall inside. Scope is checked first: a command run outside every listed scope is denied even if that exact command is on the allow list.

**Network** rules match on host.

Paths can be relative, absolute, or `~`-prefixed, and support `*` (single segment) and `**` (any depth) globs.

## The approval flow (`ask` rules)

A rule marked `ask` doesn't resolve to allow or deny on its own, it needs a decision made at runtime by an approval handler.

```rust
use fence::{ApprovalDecision, Fence};

let fence = Fence::load(".fence")
    .expect("failed to load policy")
    .with_approval_handler(|request| {
        // show the request to a human, a log, a prompt, whatever fits
        println!("Approve: {request}?");
        ApprovalDecision::Approved // or ApprovalDecision::Denied
    });
```

`ApprovalHandler` is a plain trait, so a closure or a struct with its own state both work. The handler is only ever consulted for a rule explicitly marked `ask`, it's never given the chance to override a `deny`, and whatever it approves is exactly the operation that was evaluated, nothing about the request can be substituted on the way through.

If no handler is registered, an operation that hits an `ask` rule returns `FenceOperationError::Ask`, carrying the request that needed a decision, so the failure is loud and specific rather than silently doing nothing:

```rust
match fence.write(path, content) {
    Ok(()) => println!("write succeeded"),
    Err(err) => println!("{err}"), // e.g. "policy marks `...` as ask, but no approval handler is configured..."
}
```

## Errors

`Fence::load` returns `FenceError`, either `Io` (couldn't read the policy file) or `Parse` (the `.fence` file didn't parse).

Every guarded operation (`read`, `write`, `delete`, `execute`, `connect`) returns `FenceOperationError`, and `Fence::load` returns `FenceError`. Both implement `Display` and `std::error::Error`, so they compose with `?` in your own functions.
## Examples

A runnable demo lives in `examples/playground.rs`, reading, writing, and deleting a file against a real `.fence` policy, with a terminal prompt for anything marked `ask`.

```bash
cargo run --example playground
```

## Development

```bash
cargo check
cargo test
```

## Current limits

Fence is a library-level enforcement API. It controls operations performed through the `Fence` API; it does not prevent an application from directly using `std::fs`, `std::process`, networking APIs, or other libraries to bypass Fence.

Path authorization is currently based on normalized paths and patterns rather than OS-level sandboxing.