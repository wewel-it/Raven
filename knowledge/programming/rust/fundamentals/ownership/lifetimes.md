---
title: "Lifetimes in Rust"
language: "rust"
category: "fundamentals"
topic: "lifetimes"
tags: ["rust","lifetimes"]
version: "1.0"
difficulty: "intermediate"
source: "local"
last_updated: "2026-07-22T00:00:00Z"
---

# Lifetimes

Summary

Lifetimes annotate how long references are valid.

Explanation

- Use lifetime annotations when references in function signatures might have different scopes.

Example

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
```

Notes

- The compiler infers lifetimes in many common cases.

References

- The Rust Book: Lifetimes
