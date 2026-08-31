# Fence Documentation

Full reference for the Fence crate, based on the current codebase. For a quick pitch and a policy-file walkthrough, see README.md. This document goes deeper: every public type, every method, the exact grammar the parser accepts, and the real behavior of path resolution.

## Crate layout

```
src/
  lib.rs                // re-exports the primary API
  approval.rs            // ApprovalDecision, ApprovalHandler, and FenceRequest's Display impl
  fence.rs                // Fence, FenceError, FenceOperationError
  engine/
    mod.rs                 // re-exports
    decision.rs             // Decision
    request.rs               // FenceRequest, Resource, Operation, Target
  policy/
    mod.rs                    // re-exports model::* and parser::*
    model.rs                    // Policy and its sub-structs, PathPattern, HostPattern
    parser.rs                    // parse(), ParseError
    evaluate.rs                    // Policy::evaluate, the decision algorithm
    matcher.rs                      // PathPattern/HostPattern matching, host_matches()
    path.rs                          // path resolution and normalization helpers
```

Every module here is declared `pub mod`, so nothing is private at the module level, only individual items without `pub` are hidden. In practice this gives two tiers of API: a small, intentional surface re-exported at the crate root, and a much larger surface reachable by spelling out the full module path.

## Public API surface

### Re-exported at the crate root

From `lib.rs`:

```rust
pub use approval::{ApprovalDecision, ApprovalHandler};
pub use engine::{Decision, FenceRequest, Operation, Resource, Target};
pub use fence::{Fence, FenceError, FenceOperationError};
```

This is the intended entry point: `fence::Fence`, `fence::FenceError`, `fence::FenceOperationError`, `fence::Decision`, `fence::FenceRequest`, `fence::Operation`, `fence::Resource`, `fence::Target`, `fence::ApprovalDecision`, `fence::ApprovalHandler`. Everything a typical consumer needs is in this list, and everything below in "`Fence`", "The approval flow", and "Errors" refers to it.

One naming note: the module holding `Fence` is itself named `fence.rs`, inside a crate also named `fence`. Without the re-export above, the real path to the type would be `fence::fence::Fence`. That `pub use` isn't just convenience, it's what makes `fence::Fence` work at all.

### Reachable, but not re-exported

Because every module is `pub mod`, the parser, matcher, and path-resolution internals are technically public too, just reached by full path instead of the crate root:

```rust
fence::policy::{Policy, FilesystemPolicy, FilesystemRules, ProcessPolicy, NetworkPolicy, PathPattern, HostPattern}
fence::policy::{parse, ParseError}
fence::policy::matcher::host_matches
fence::policy::path::{expand_home, normalize_path, resolve_runtime_path, normalize_runtime_path, normalize_pattern}
fence::engine::decision::Decision                                    // same type as fence::Decision
fence::engine::request::{FenceRequest, Operation, Resource, Target}  // same types as the root re-exports
```

`Policy` and `ParseError` are reachable directly under `fence::policy::` because `policy/mod.rs` re-exports `model::*` and `parser::*`. `host_matches` and the `path` helpers need the extra `matcher::`/`path::` segment, since those two modules aren't wildcard re-exported one level up.

These exist for building tooling around Fence (a `.fence` linter, a policy visualizer, a standalone path matcher) without going through `Fence` itself. The "Advanced: policy internals" section below documents each of these. Most code that just wants to guard its own filesystem/process/network calls never needs this layer, `Fence` is the whole interface for that.

## The `.fence` policy format

A `.fence` file is read line by line, 1-indexed for error messages.

- Everything from a `#` to the end of the line is stripped as a comment, so `#` can't appear inside a value (a path, for instance) without being treated as the start of a comment.
- Blank lines, after comment-stripping and trimming, are skipped.
- A line of the exact form `[filesystem]`, `[process]`, or `[network]` switches the active section. Any other bracketed line is a parse error (`unknown section`).
- Every other non-blank line is a rule, split into up to three whitespace-separated fields: `<action> <kind> <values...>`. The third field is everything remaining after the second, not re-split on whitespace, so a values list can contain spaces (e.g. around commas) without breaking the split.
- A rule line encountered before any `[section]` header is a parse error (`rule found before a section`).

### `[filesystem]`

```
<allow|ask|deny> <read|write|delete> <path>[, <path>...]
```

`values` is split on commas, each entry trimmed, empty entries dropped, at least one value required. Each becomes a `PathPattern` appended to the matching action/kind list (e.g. `ask` + `write` appends to `filesystem.ask.write`).

### `[process]`

Two rule shapes:

```
allow scope <path>[, <path>...]
<allow|ask|deny> command <name>[, <name>...]
```

`scope` only accepts `allow` as its action, `ask scope ...` or `deny scope ...` is a parse error (`scope can only use allow`). Scope values become `PathPattern`s appended to `process.scope`.

`command` values become plain `String`s (not path patterns), later matched by exact string equality against the command name in a request, not by glob.

### `[network]`

```
<allow|ask|deny> host <pattern>[, <pattern>...]
```

Values become `HostPattern`s.

Any unrecognized action or kind word, or a rule missing its values, produces a `ParseError` naming the line number and a short message (`unknown action: ...`, `missing filesystem path`, etc.).

## Path patterns and resolution

### Matching semantics

- `*` matches exactly one path segment, any content.
- `**` matches zero or more segments. The matcher first tries consuming zero, then backtracks to consume one segment at a time and retries, so `projects/**` matches `projects` itself as well as anything nested under it, at any depth.
- Both sides of a comparison, the pattern and the path being checked, are resolved to absolute, normalized form first (see below), then split on `/` and compared segment by segment.
- Host patterns: `*` matches any host; a `*.suffix` prefix matches only subdomains of `suffix` (`api.example.com` matches `*.example.com`, but bare `example.com` does not); anything else must match exactly.

### Resolution mechanics

- `expand_home(path)`: a bare `~` becomes `$HOME`; a `~/...` prefix becomes `$HOME/...`; anything else passes through unchanged. Requires the `HOME` environment variable to be set, returns an `io::Error` if it isn't.
- `normalize_path(path)`: purely lexical, resolves `.` (dropped) and `..` (pops the previous segment), no filesystem access, so it works for paths that don't exist yet. A leading `..` past the root doesn't error, it's dropped once there's nothing left to pop.
- `resolve_runtime_path(path, base)`: expands `~`, joins onto `base` if the result isn't already absolute, then lexically normalizes. This is what every `Fence` operation calls internally, with `base` set to the `Fence`'s root.
- `normalize_runtime_path(path)`: the same, but resolves against `std::env::current_dir()` instead of an explicit base. Not called anywhere inside `Fence` itself, a standalone convenience for code that wants "resolve the way the OS would from here."
- `normalize_pattern(pattern, base)`: the pattern-string equivalent, used internally before a pattern is compared against a path.

### A note on traversal

Resolution is lexical only. It doesn't touch the filesystem and doesn't clamp the result back inside `base`, a relative path with enough `../` segments can resolve to somewhere outside a `Fence`'s root. What keeps this safe in practice is that policy rules match against the *final resolved path*, not the string a caller passed in: if `../../../etc/passwd` resolves outside everything your `allow` patterns cover, it simply won't match any of them, and the default-deny fallback catches it.

The practical implication: the security boundary comes entirely from how narrowly your `allow`/`ask` patterns are scoped. A broad pattern like `allow read /**` gives a traversal attempt somewhere to land; a narrow one like `allow read ./playground/**` doesn't.

## `Fence`

### `Fence::load`

```rust
pub fn load(path: impl AsRef<Path>) -> Result<Self, FenceError>
```

Reads and parses a `.fence` policy file at `path`. The path must have a `.fence` extension. The parent directory of `path` is canonicalized and becomes the `Fence`'s root, the base against which relative patterns in the policy and relative paths passed to `read`, `write`, `delete`, and `execute` are resolved.

Because the parent directory is canonicalized, it must actually exist on disk.

Returns `FenceError::InvalidPolicyFile` when the path does not have a `.fence` extension, `FenceError::Io` if the file cannot be read or its parent cannot be canonicalized, and `FenceError::Parse` if the policy contents cannot be parsed.

### `Fence::with_approval_handler`

```rust
pub fn with_approval_handler(self, handler: impl ApprovalHandler + 'static) -> Self
```

Builder-style, consumes and returns `Self`. Registers a handler consulted whenever policy resolves to `Ask`. Without one, `Ask` decisions surface as `FenceOperationError::Ask`.

### `Fence::check`

```rust
pub fn check(&self, request: &FenceRequest) -> Decision
```

Runs policy evaluation only, no I/O, no approval handler consulted. Build a request with `FenceRequest::filesystem(...)`, `::process(...)`, or `::network(...)` and inspect what the policy would say before deciding whether to call the real operation.

### `Fence::read`, `write`, `delete`

```rust
pub fn read(&self, path: impl AsRef<Path>) -> Result<Vec<u8>, FenceOperationError>
pub fn write(&self, path: impl AsRef<Path>, content: impl AsRef<[u8]>) -> Result<(), FenceOperationError>
pub fn delete(&self, path: impl AsRef<Path>) -> Result<(), FenceOperationError>
```

Each resolves `path` against the `Fence`'s root via `resolve_runtime_path`, evaluates the resolved path against the matching `[filesystem]` list, and only then performs the real `std::fs` call, using that same resolved path for both the check and the actual operation.

### `Fence::execute`

```rust
pub fn execute<I, S>(&self, command: impl Into<String>, args: I, cwd: impl AsRef<Path>) -> Result<std::process::Output, FenceOperationError>
where I: IntoIterator<Item = S>, S: Into<String>
```

Resolves `cwd` against the root, checks `[process]` (scope first, independently, then the command name against allow/ask/deny), and on success spawns via `std::process::Command`, returning its captured `Output`. `args` are passed through to the spawned process untouched, they are not part of what policy evaluates.

### `Fence::connect`

```rust
pub fn connect(&self, host: impl Into<String>, port: u16) -> Result<std::net::TcpStream, FenceOperationError>
```

Checks `host` against `[network]`, and on success opens a `std::net::TcpStream` to `(host, port)`. Unlike the filesystem and process methods, nothing here is resolved against the `Fence` root, a host string has no notion of a base directory.

## The approval flow

### `ApprovalDecision`

```rust
pub enum ApprovalDecision {
    Approved,
    Denied,
}
```

### `ApprovalHandler`

```rust
pub trait ApprovalHandler: Send + Sync {
    fn approve(&self, request: &FenceRequest) -> ApprovalDecision;
}
```

There's a blanket implementation for any `Fn(&FenceRequest) -> ApprovalDecision + Send + Sync`, covering closures and plain function pointers, so most cases don't need a named type. A struct implementing the trait directly is useful when the decision needs to carry state across calls (a cache, a counter, a lock).

### Behavior guarantees

- The handler is only ever consulted when policy resolves to `Ask`. `Deny` returns before the handler is reached, no handler can override an explicit deny.
- The handler receives the exact `FenceRequest` that was evaluated and returns only `Approved`/`Denied`, no new parameters flow back in, so approving a request can't be used to substitute a different path, command, or host than the one actually checked.
- With no handler registered, `Ask` surfaces as `FenceOperationError::Ask(request)` rather than defaulting either way.

### `FenceRequest`'s `Display` impl

Lives in `engine/request.rs`, next to the rest of the type's definition:

```rust
impl std::fmt::Display for FenceRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.operation, &self.target) {
            (Operation::Read, Target::Path(path)) => write!(f, "read {}", path.display()),
            (Operation::Write, Target::Path(path)) => write!(f, "write {}", path.display()),
            (Operation::Delete, Target::Path(path)) => write!(f, "delete {}", path.display()),
            (Operation::Execute, Target::Process { command, args, cwd }) => {
                if args.is_empty() {
                    write!(f, "run `{command}` in {}", cwd.display())
                } else {
                    write!(f, "run `{command} {}` in {}", args.join(" "), cwd.display())
                }
            }
            (Operation::Connect, Target::Network { host, port }) => write!(f, "connect to {host}:{port}"),
            _ => write!(f, "{:?} on {:?}", self.operation, self.target),
        }
    }
}
```

It formats per operation. For example, a request prints as
`read /home/user/project/playground/test.txt`,
``run `cargo test` in /home/user/project``, or
`connect to 127.0.0.1:8080`.

The final `_` arm is a fallback for a mismatched
resource/operation/target combination. It is technically
constructible since all of `FenceRequest`'s fields are public,
though nothing in the crate builds one that way, so it isn't
reachable in normal use.

## Errors

### `FenceError`

```rust
pub enum FenceError {
    Io(std::io::Error),
    Parse(ParseError),
}
```

Returned by `Fence::load`. Implements `Display` and `std::error::Error`. `Io` and `Parse` expose their underlying errors through `source()`, while `InvalidPolicyFile` indicates that the supplied path is not a `.fence` policy file.

### `FenceOperationError`

```rust
pub enum FenceOperationError {
    Denied,
    Ask(FenceRequest),
    Io(std::io::Error),
}
```

Returned by `read`, `write`, `delete`, `execute`, `connect`.

- `Denied`, policy resolved to Deny, or an approval handler returned `ApprovalDecision::Denied`.
- `Ask(request)`, policy resolved to Ask and no approval handler is registered.
- `Io(err)`, the policy check passed but the underlying `std::fs`/`std::process`/`std::net` call itself failed.

Implements `Display` (a per-variant message, including a pointer to `.with_approval_handler(...)` for `Ask`) and `std::error::Error` (exposing `Io`'s inner error through `source()`).

### `ParseError`

```rust
pub struct ParseError {
    pub line: usize,
    pub message: String,
}
```

Both fields are public, so a caller can inspect the line number and message directly instead of only formatting them. Implements `Display` (`line {line}: {message}`) and `std::error::Error`, with no `source()` override since it's the root cause itself, not a wrapper around another error.

## Advanced: policy internals

Everything below bypasses `Fence` entirely. Useful for tooling (validating a `.fence` file without touching disk, matching a path against a single pattern, building a policy programmatically), not needed for normal usage.

### Parsing without a `Fence`

```rust
use fence::policy::parse;

let policy = parse(".fence file contents as a &str")?; // Result<Policy, ParseError>
```

### `Policy` and evaluating requests directly

```rust
use fence::policy::Policy;
use fence::{FenceRequest, Operation};
use std::path::Path;

let policy: Policy = /* parsed, or built by hand: every field is public and Policy derives Default */;
let request = FenceRequest::filesystem(Operation::Read, "/some/path");
let decision = policy.evaluate(&request, Path::new("/some/base"));
```

`Policy`, `FilesystemPolicy`, `FilesystemRules`, `ProcessPolicy`, `NetworkPolicy` all derive `Default` and have entirely public fields, so a policy can be constructed in code instead of parsed from a file. There is currently no public constructor that builds a `Fence` from an in-memory `Policy`, `Fence::load` is the only way to build one, and it always parses from a file path. Calling `.evaluate()` on a `Policy` directly is the only way to use policy logic without a file on disk.

### Matching a single pattern

```rust
use fence::policy::{PathPattern, HostPattern};
use fence::policy::matcher::host_matches;
use std::path::Path;

PathPattern("./playground/**".into()).matches_from(Path::new("./playground/test.txt"), Path::new("/home/user/project"));
HostPattern("*.example.com".into()).matches("api.example.com");
host_matches("*.example.com", "api.example.com"); // same thing, as a free function
```

`PathPattern::matches` resolves relative to `std::env::current_dir()`; `PathPattern::matches_from` takes an explicit base, the version `Fence` itself uses internally.

## Testing

Three integration test files, run with `cargo test`:

- `tests/fence.rs`, exercises the `Fence` struct's public methods end to end (load, check, read, write, delete, execute, connect, and the approval flow through each).
- `tests/policy_evaluation.rs`, exercises `Policy::evaluate` directly against hand-built policies, without going through the parser or `Fence`.
- `tests/policy_parser.rs`, exercises `parse()` directly, valid and invalid `.fence` syntax.

## Known limitations

Fence enforces only what goes through the `Fence` API itself, it doesn't stop code that reaches `std::fs`, `std::process`, `std::net`, or another library directly.

Path authorization is pattern matching on normalized paths and patterns, not OS-level sandboxing, see "A note on traversal" above for exactly what that does and doesn't protect against.