use fence::Fence;
use std::fs;

fn main() {
    let file = "examples/playground/test.txt";

    let fence = Fence::load("examples/playground.fence").expect("failed to load playground policy");
    println!("Fence playground loaded successfully.");

    // Read
    let content = fence.read(file).expect("read was denied or failed");
    println!("Read: {}", String::from_utf8_lossy(&content));

    // Write
    fence
        .write(file, "Hello from Fence!")
        .expect("write was denied or failed");
    println!("Write succeeded.");

    // Read again
    let content = fence.read(file).expect("read was denied or failed");
    println!("Read after write: {}", String::from_utf8_lossy(&content));

    // Delete
    fence.delete(file).expect("delete was denied or failed");
    println!("Delete succeeded.");

    // Verify the file is actually gone.
    assert!(!fs::exists(file).expect("failed to check file existence"));
    println!("File successfully deleted. Create/write test.txt again before the next run.");
}
