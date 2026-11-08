//! A lightweight cursor over slices in [Eytzinger layout].
//! It moves through the elements in sorted order, one step at a time, in either direction.
//!
//! Eytzinger layout stores sorted values as an implicit binary tree,
//! which makes binary search cache-friendly.
//!
//! [Eytzinger layout]: https://algorithmica.org/en/eytzinger

#![warn(clippy::all, clippy::cargo, clippy::pedantic)]
#![no_std]

/// A cyclic cursor over a slice whose elements are in Eytzinger layout.
///
/// # Ghost position
///
/// The cursor has one extra position that points to no element.
/// It sites between the last and the first position.
///
/// # Examples
///
/// Fill a slice with the sorted values in Eytzinger layout.
///
/// ```
/// # use eytzinger_cursor::Cursor;
/// #
/// let n = 10;
///
/// let mut slice = vec![!0; n];
/// let mut value = 0;
///
/// let mut cursor = Cursor::new(n);
/// while let Some(i) = cursor.next() {
///     slice[i] = value;
///     value += 1;
/// }
///
/// assert_eq!(slice, vec![6, 3, 8, 1, 5, 7, 9, 0, 2, 4])
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cursor {
    len: usize,
    // 1-based indexing
    ptr: usize,
}

impl Cursor {
    /// Create a new cursor for a slice of `len` elements.
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

    /// Returns the length of the slice that the cursor assumes.
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

    /// Returns the current index in the slice, or `None` if the cursor is at the ghost position.
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

    /// Moves the cursor to the element at index `i` in the slice.
    ///
    /// If the given index `i` is out of bounds, the cursor moves to the ghost.
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
    /// assert!(cursor.set(5));
    /// assert_eq!(cursor.get(), Some(5));
    ///
    /// assert!(!cursor.set(100));
    /// assert_eq!(cursor.get(), None);
    /// ```
    pub const fn set(&mut self, i: usize) -> bool {
        self.ptr = if i < self.len { i + 1 } else { 0 };
        self.ptr > 0
    }

    /// Moves the cursor to the node that stores `nth` smallest value
    /// and returns the index in the slice.
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

    /// Moves the cursor to the next element and returns the index in the slice.
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
            // go up through left-child edges, then up one right-child edge
            ptr >>= ptr.trailing_ones();
            ptr >>= 1;
        }
        self.ptr = ptr;

        self.get()
    }

    /// Moves the cursor to the previous element and returns the index in the slice.
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
            // go down the right-most node in the left subtree
            ptr = !(!ptr << (ptr.leading_zeros() - len.leading_zeros()));
            ptr >>= if ptr > len { 1 } else { 0 };
        } else {
            // go up through right-child edges, then up one left-child edge
            ptr >>= ptr.trailing_zeros();
            ptr >>= 1;
        }
        self.ptr = ptr;

        self.get()
    }
}
