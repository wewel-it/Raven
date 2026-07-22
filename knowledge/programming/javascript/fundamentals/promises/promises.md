---
title: "JavaScript Promises Basics"
language: "javascript"
category: "asynchronous"
topic: "promises"
tags: ["javascript", "promises", "async"]
version: "1.0"
difficulty: "intermediate"
source: "local"
last_updated: "2026-07-21T00:00:00Z"
---

# JavaScript Promises Basics

## Summary

Promises provide a standard way to handle asynchronous operations in JavaScript.

## Explanation

A `Promise` represents a value that may be available now, later, or never. It has `then`, `catch`, and `finally` handlers.

## Example Code

```javascript
const promise = new Promise((resolve, reject) => {
  setTimeout(() => resolve('done'), 1000);
});

promise.then((value) => {
  console.log(value);
});
```

## Notes

- Use `async`/`await` for cleaner promise-based code.
- Always handle rejections with `catch`.
- Promises are chainable.

## References

- https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Using_promises
