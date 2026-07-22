---
title: "Python Asyncio Basics"
language: "python"
category: "concurrency"
topic: "asyncio"
tags: ["python", "asyncio", "concurrency", "await"]
version: "1.0"
difficulty: "intermediate"
source: "local"
last_updated: "2026-07-21T00:00:00Z"
---

# Python Asyncio Basics

## Summary

`asyncio` provides asynchronous I/O and cooperative multitasking in Python using `async`/`await` syntax.

## Explanation

An `async` function returns a coroutine. The event loop schedules coroutines and manages task execution.

## Example Code

```python
import asyncio

async def greet():
    print("Hello from asyncio")

async def main():
    await greet()

asyncio.run(main())
```

## Notes

- Use `await` to suspend coroutine execution until a result is available.
- `asyncio` is single-threaded by default.
- Tasks can run concurrently but not in parallel on one thread.

## References

- https://docs.python.org/3/library/asyncio.html
