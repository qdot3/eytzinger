//! A lightweight cursor over an [Eytzinger array] (an implicit binary search tree).
//!
//! # Why Eytzinger
//!
//! In this layout, the elements that a binary search may visit `n` steps ahead
//! live in consecutive space.
//! So we can prefetch them with only a few cache line loads, and this hides memory latency.
//!
//! [Eytzinger array]: https://algorithmica.org/en/eytzinger
//!
//! # Example
//!
//! Constructs an Eytzinger array.
//!
//! ```
//! # use eytzinger_cursor::Cursor;
//! #
//! let mut sorted = vec![2, 3, 5, 7, 11, 13, 17];
//!
//! let mut n = sorted.len();
//! let mut cursor = Cursor::new(n);
//!
//! let mut arr = vec![!0; n];
//! let mut val = 0;
//! while let Some(ptr) = cursor.prev() {
//!     arr[ptr] = sorted.pop().unwrap();
//! }
//!
//! assert_eq!(arr, [
//!          7,
//!       3,    13,
//!     2, 5, 11, 17,
//! ]);
//! ```
#![warn(clippy::all, clippy::cargo, clippy::pedantic)]
#![no_std]

/// A lightweight cyclic cursor that walks an Eytzinger array.
///
/// This `Cursor` can move forward and backward in constant time.
/// It can also jump to the position where the n-th smallest value lives.
///
/// By using `Cursor`, an Eytzinger array can be constructed from sorted values in linear time
/// and vice versa.
///
/// # Ghost position
///
/// The cursor has one extra position that points to no element.
/// It sits between the last and the first position.
///
/// # Examples
///
/// ```
/// # use eytzinger_cursor::Cursor;
/// #
/// let n = 10;
/// let mut cursor = Cursor::new(n);
///
/// let mut val = 0;
/// let mut arr = vec![!0; n];
/// while let Some(i) = cursor.next() {
///     arr[i] = val;
///     val += 1;
/// }
///
/// let mut rra = vec![!0; n];
/// while let Some(i) = cursor.prev() {
///     val -= 1;
///     rra[i] = val;
/// }
///
/// assert_eq!(arr, rra)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cursor {
    len: usize,
    // 1-based indexing
    ptr: usize,
}

impl Cursor {
    /// Creates a new cursor for an array of `len` elements.
    ///
    /// The cursor starts at the ghost.
    ///
    /// # Panics
    ///
    /// `len` must be **no more** than `isize::MAX`.
    ///
    /// # Time complexity
    ///
    /// O(1)
    ///
    /// # Examples
    ///
    /// ```
    /// # use eytzinger_cursor::Cursor;
    /// #
    /// let cursor = Cursor::new(10);
    /// ```
    #[must_use]
    pub const fn new(len: usize) -> Self {
        assert!(len <= isize::MAX.cast_unsigned());

        Self { len, ptr: 0 }
    }

    /// Moves the cursor to the ghost.
    ///
    /// # Time complexity
    ///
    /// O(1)
    ///
    /// # Examples
    ///
    /// ```
    /// # use eytzinger_cursor::Cursor;
    /// #
    /// let mut cursor = Cursor::new(10);
    ///
    /// cursor.nth(5);
    /// assert_eq!(cursor.get(), Some(4));
    ///
    /// cursor.reset();
    /// assert_eq!(cursor.get(), None);
    /// ```
    pub const fn reset(&mut self) {
        self.ptr = 0
    }

    /// Returns the length of the array that the cursor assumes.
    ///
    /// # Time complexity
    ///
    /// O(1)
    ///
    /// # Examples
    ///
    /// ```
    /// # use eytzinger_cursor::Cursor;
    /// #
    /// let mut cursor = Cursor::new(10);
    /// assert_eq!(cursor.len(), 10);
    /// ```
    #[must_use]
    #[allow(clippy::len_without_is_empty)]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns the current index in the array, or `None` if the cursor is at the ghost position.
    ///
    /// # Time complexity
    ///
    /// O(1)
    ///
    /// # Examples
    ///
    /// ```
    /// # use eytzinger_cursor::Cursor;
    /// #
    /// let mut cursor = Cursor::new(10);
    /// // points to ghost
    /// assert_eq!(cursor.get(), None);
    ///
    /// cursor.next();
    /// assert_eq!(cursor.get(), Some(7));
    /// ```
    #[must_use]
    pub const fn get(&self) -> Option<usize> {
        self.ptr.checked_sub(1)
    }

    /// Moves the cursor to the element at index `i` in the array.
    ///
    /// If `i` is out of bounds, the cursor moves to the ghost.
    ///
    /// # Time complexity
    ///
    /// O(1)
    ///
    /// # Examples
    ///
    /// ```
    /// # use eytzinger_cursor::Cursor;
    /// #
    /// let mut cursor = Cursor::new(10);
    ///
    /// cursor.set(5);
    /// assert_eq!(cursor.get(), Some(5));
    ///
    /// cursor.set(100);
    /// assert_eq!(cursor.get(), None);
    /// ```
    pub const fn set(&mut self, i: usize) {
        self.ptr = if i < self.len { i + 1 } else { 0 };
    }

    /// Moves the cursor to the node that stores the `nth` smallest value
    /// and returns the index in the array.
    ///
    /// If `nth` is out of bounds, the cursor moves to the ghost.
    ///
    /// See [`get`](Self::get) for the return value.
    ///
    /// # Time complexity
    ///
    /// O(log N)
    ///
    /// # Examples
    ///
    /// ```
    /// # use eytzinger_cursor::Cursor;
    /// #
    /// for (n, layout) in [
    ///     (0, vec![]),
    ///     (1, vec![0]),
    ///     (7, vec![3, 1, 5, 0, 2, 4, 6]),
    ///     (10, vec![6, 3, 8, 1, 5, 7, 9, 0, 2, 4]),
    /// ] {
    ///     let mut cursor = Cursor::new(n);
    ///
    ///     for i in 0..n {
    ///         assert_eq!(layout[cursor.nth(i).unwrap()], i)
    ///     }
    ///     // invalid input
    ///     assert_eq!(cursor.nth(100), None);
    /// }
    /// ```
    pub fn nth(&mut self, mut nth: usize) -> Option<usize> {
        use core::cmp::Ordering;

        if nth < self.len {
            nth += 1;
            // move the cursor to the root
            let mut ptr = 1;

            let height = self.len.ilog2();
            let mut upper = (1 << height) - 1;
            let mut ground = self.len - upper;
            for _ in 0..height {
                upper /= 2;
                // size of left subtree
                let l = upper + ground.min(upper + 1);

                match nth.cmp(&(l + 1)) {
                    // go down left
                    Ordering::Less => {
                        ptr *= 2;
                        ground = ground.min(upper + 1);
                    }
                    // go down right
                    Ordering::Greater => {
                        ptr = ptr * 2 + 1;
                        nth -= l + 1;
                        ground -= ground.min(upper + 1);
                    }
                    Ordering::Equal => break,
                }
            }

            self.ptr = ptr;
        } else {
            self.ptr = 0;
        }

        self.get()
    }

    /// Moves the cursor to the next element and returns the index in the array.
    ///
    /// See [`get`](Self::get) for the return value.
    ///
    /// # Time complexity
    ///
    /// O(1)
    ///
    /// # Examples
    ///
    /// ```
    /// # use eytzinger_cursor::Cursor;
    /// #
    /// for (n, layout) in [
    ///     (0, vec![]),
    ///     (1, vec![0]),
    ///     (7, vec![3, 1, 5, 0, 2, 4, 6]),
    ///     (10, vec![6, 3, 8, 1, 5, 7, 9, 0, 2, 4]),
    /// ] {
    ///     let mut cursor = Cursor::new(n);
    ///
    ///     for i in 0..n {
    ///         assert_eq!(layout[cursor.next().unwrap()], i)
    ///     }
    ///     // points to ghost
    ///     assert_eq!(cursor.next(), None);
    /// }
    /// ```
    pub const fn next(&mut self) -> Option<usize> {
        let Self { len, mut ptr } = *self;
        ptr = ptr * 2 + 1;
        if ptr <= len {
            // go down to the left-most leaf in the right subtree
            ptr <<= ptr.leading_zeros() - len.leading_zeros();
            ptr >>= if ptr > len { 1 } else { 0 };
        } else {
            // go up through right-child edges, then up one left-child edge
            ptr >>= ptr.trailing_ones();
            ptr >>= 1;
        }
        self.ptr = ptr;

        self.get()
    }

    /// Moves the cursor to the previous element and returns the index in the array.
    ///
    /// See [`get`](Self::get) for the return value.
    ///
    /// # Time complexity
    ///
    /// O(1)
    ///
    /// # Examples
    ///
    /// ```
    /// # use eytzinger_cursor::Cursor;
    /// #
    /// for (n, layout) in [
    ///     (0, vec![]),
    ///     (1, vec![0]),
    ///     (7, vec![3, 1, 5, 0, 2, 4, 6]),
    ///     (10, vec![6, 3, 8, 1, 5, 7, 9, 0, 2, 4]),
    /// ] {
    ///     let mut cursor = Cursor::new(n);
    ///
    ///     for i in 0..n {
    ///         assert_eq!(layout[cursor.prev().unwrap()], n - i - 1)
    ///     }
    ///     // points to ghost
    ///     assert_eq!(cursor.prev(), None);
    /// }
    /// ```
    pub const fn prev(&mut self) -> Option<usize> {
        let Self { len, mut ptr } = *self;
        ptr *= 2;
        if ptr <= len {
            // go down to the right-most node in the left subtree
            ptr = !(!ptr << (ptr.leading_zeros() - len.leading_zeros()));
            ptr >>= if ptr > len { 1 } else { 0 };
        } else {
            // go up through left-child edges, then up one right-child edge
            ptr >>= ptr.trailing_zeros();
            ptr >>= 1;
        }
        self.ptr = ptr;

        self.get()
    }
}
