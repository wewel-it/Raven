---
title: "Rust Borrowing Overview"
language: "rust"
category: "ownership"
topic: "borrowing"
tags: ["rust", "ownership", "borrowing", "memory"]
version: "1.0"
difficulty: "intermediate"
source: "local"
last_updated: "2026-07-21T00:00:00Z"
---

# Rust Borrowing Overview

## Summary

Borrowing is a core Rust ownership concept that allows references to a value without taking ownership. It enables safe shared and mutable access patterns.

## Explanation

In Rust, references can be immutable (`&T`) or mutable (`&mut T`). The borrow checker enforces rules so that data races and invalid memory access are prevented.

## Example Code

```rust
fn main() {
    let value = String::from("hello");
    let borrowed = &value;
    println!("{}", borrowed);
}
```

## Notes

- Immutable borrows can be used many times simultaneously.
- Only one mutable borrow is allowed at a time.
- References must always be valid.

## References

- https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html
