# Fence

Fence is a policy-based security layer for controlling filesystem, process, and network access.

Policies are written in `.fence` files:

```text
[filesystem]
allow read ~/projects/**
deny delete ~/projects/**/node_modules/**

[process]
allow command git, cargo
deny command bash

[network]
allow host api.github.com
deny host *
```

Fence evaluates requests and returns:

```text
Allow
Ask
Deny
```

The default behavior is **deny** when no rule allows an operation.

## Status

Currently implemented:

* Filesystem policy evaluation
* Process policy evaluation
* Network policy evaluation
* Path and host matching
* Initial `.fence` parser

Still being developed:

* Complete `.fence` parsing
* `Fence::load()`
* Runtime enforcement
* `Ask` handling

## Development

```bash
cargo check
cargo test
```

Fence is currently a policy evaluator, not yet a complete sandbox.
