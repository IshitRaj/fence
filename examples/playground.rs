//! Minimal end-to-end demo of Fence: load a policy, wire up an approval
//! handler for anything marked `ask`, then read, write, and delete a file,
//! logging whatever the policy decides rather than crashing on it.
//!
//! Run with: cargo run --example playground

use fence::{ApprovalDecision, Fence};
use std::io::{self, Write};

fn main() {
    let file = "playground/test.txt";

    let fence = Fence::load("examples/playground.fence")
        .expect("failed to load fence policy")
        .with_approval_handler(prompt_for_approval);

    println!("Fence playground loaded successfully.\n");

    match fence.read(file) {
        Ok(content) => println!("[read] succeeded -> {}", String::from_utf8_lossy(&content)),
        Err(err) => println!("[read] {err}"),
    }

    match fence.write(file, "Hello from Fence!") {
        Ok(()) => println!("[write] succeeded"),
        Err(err) => println!("[write] {err}"),
    }

    match fence.read(file) {
        Ok(content) => println!(
            "[read after write] succeeded -> {}",
            String::from_utf8_lossy(&content)
        ),
        Err(err) => println!("[read after write] {err}"),
    }

    match fence.delete(file) {
        Ok(()) => println!("[delete] succeeded"),
        Err(err) => println!("[delete] {err}"),
    }
}

/// Prompts in the terminal whenever a policy rule is marked `ask`.
fn prompt_for_approval(request: &fence::FenceRequest) -> ApprovalDecision {
    print!("Approve: {request}? [y/N] ");
    io::stdout().flush().ok();

    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();

    if input.trim().eq_ignore_ascii_case("y") {
        ApprovalDecision::Approved
    } else {
        ApprovalDecision::Denied
    }
}
