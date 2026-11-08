# eytzinger-cursor

A lightweight cursor over slices in [Eytzinger layout].
It moves through the elements in sorted order, one step at a time, in either direction.

Eytzinger layout stores sorted values as an implicit binary tree,
which makes binary search cache-friendly.

[Eytzinger layout]: https://algorithmica.org/en/eytzinger

## Examples

Fill a slice with the sorted values in Eytzinger layout.

```rust
use eytzinger_cursor::Cursor;

let n = 10;

let mut slice = vec![!0; n];
let mut value = 0;

let mut cursor = Cursor::new(n);
while let Some(i) = cursor.next() {
    slice[i] = value;
    value += 1;
}

assert_eq!(slice, vec![6, 3, 8, 1, 5, 7, 9, 0, 2, 4])
```

## License

MIT
