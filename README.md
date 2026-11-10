# eytzinger-cursor

A lightweight cursor over an [Eytzinger array] (an implicit binary search tree).

## Why Eytzinger

In this layout, the elements that a binary search may visit `n` steps ahead
live in consecutive space.
So we can prefetch them with only a few cache line loads, and this hides memory latency.

[Eytzinger array]: https://algorithmica.org/en/eytzinger

## Example

Constructs an Eytzinger array.

```rust
#
let mut sorted = vec![2, 3, 5, 7, 11, 13, 17];

let mut n = sorted.len();
let mut cursor = Cursor::new(n);

let mut arr = vec![!0; n];
let mut val = 0;
while let Some(ptr) = cursor.prev() {
    arr[ptr] = sorted.pop().unwrap();
}

assert_eq!(arr, [
         7,
      3,    13,
    2, 5, 11, 17,
]);
```

## LICENSE

MIT
