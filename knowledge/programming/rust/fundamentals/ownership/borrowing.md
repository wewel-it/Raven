---
title: "Borrowing in Rust"
language: "rust"
category: "fundamentals"
topic: "ownership"
tags: ["rust","ownership","borrowing"]
version: "1.0"
difficulty: "intermediate"
source: "local"
last_updated: "2026-07-22T00:00:00Z"
---

# Borrowing in Rust

Summary

Rust uses borrowing to allow references to data without transferring ownership.

Explanation

- Borrowing comes in two forms: immutable (`&T`) and mutable (`&mut T`).
- The borrow checker enforces rules to prevent data races and invalid references.

Example

```rust
fn main() {
    let mut s = String::from("hello");
    let r1 = &s; // immutable borrow
    let r2 = &s; // immutable borrow
    println!("{} {}", r1, r2);

    let r3 = &mut s; // mutable borrow, not allowed while immutable borrows exist
    r3.push_str(" world");
    println!("{}", r3);
}
```

Notes

- Keep borrows short-lived and prefer immutable borrows when possible.

References

- The Rust Book: Ownership
