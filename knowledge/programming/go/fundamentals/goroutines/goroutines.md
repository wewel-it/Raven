---
title: "Go Goroutines Basics"
language: "go"
category: "concurrency"
topic: "goroutines"
tags: ["go", "concurrency", "goroutine", "async"]
version: "1.0"
difficulty: "intermediate"
source: "local"
last_updated: "2026-07-21T00:00:00Z"
---

# Go Goroutines Basics

## Summary

Goroutines are lightweight threads managed by the Go runtime. They allow concurrent execution of functions with minimal overhead.

## Explanation

A goroutine is created by prefixing a function call with `go`. The Go scheduler multiplexes goroutines onto OS threads.

## Example Code

```go
package main

import "fmt"

func sayHello() {
    fmt.Println("Hello from goroutine")
}

func main() {
    go sayHello()
    fmt.Println("Main function finished")
}
```

## Notes

- Use channels to communicate between goroutines safely.
- The main function may exit before goroutines complete.
- Goroutines are not the same as OS threads.

## References

- https://go.dev/doc/effective_go#goroutines
