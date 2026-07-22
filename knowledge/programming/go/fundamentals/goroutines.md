---
title: "Goroutines in Go"
language: "go"
category: "fundamentals"
topic: "concurrency"
tags: ["go","goroutines","concurrency"]
version: "1.0"
difficulty: "intermediate"
source: "local"
last_updated: "2026-07-22T00:00:00Z"
---

# Goroutines

Summary

Goroutines are lightweight threads managed by the Go runtime.

Explanation

- Use `go func()` to start a goroutine.
- Use channels to communicate safely between goroutines.

Example

```go
package main
import (
    "fmt"
)
func main(){
    go func(){ fmt.Println("hello from goroutine") }()
}
```

Notes

- Synchronize with channels or WaitGroups.
