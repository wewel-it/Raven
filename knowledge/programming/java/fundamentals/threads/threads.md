---
title: "Java Threads Basics"
language: "java"
category: "concurrency"
topic: "threads"
tags: ["java", "threads", "concurrency"]
version: "1.0"
difficulty: "intermediate"
source: "local"
last_updated: "2026-07-21T00:00:00Z"
---

# Java Threads Basics

## Summary

Java threads allow parallel execution of code within the same Java process. They are managed by the JVM and can share memory.

## Explanation

A thread can be created by extending `Thread` or implementing `Runnable`. Thread scheduling is handled by the operating system.

## Example Code

```java
public class ThreadExample {
    public static void main(String[] args) {
        Thread thread = new Thread(() -> System.out.println("Hello from thread"));
        thread.start();
    }
}
```

## Notes

- Use synchronization to avoid race conditions.
- Prefer higher-level concurrency utilities like `ExecutorService`.
- Threads share heap memory but have separate stacks.

## References

- https://docs.oracle.com/javase/tutorial/essential/concurrency/
