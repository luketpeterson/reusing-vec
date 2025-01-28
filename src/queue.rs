
extern crate alloc;
use alloc::vec::Vec;

use crate::*;

/// A structure similar to [`ReusingVec`](crate::ReusingVec), but with support for a
/// [`pop_front`](Self::pop_front) operation
#[derive(Clone, Default)]
pub struct ReusingQueue<T> {
    logical_start: usize,
    logical_end: usize,
    contents: Vec<T>
}

impl<T> core::fmt::Debug for ReusingQueue<T> where T: core::fmt::Debug {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "{:?}", self as &[_])
    }
}

impl<T> ReusingQueue<T> {
    /// Create a new empty vector, does not allocate until the first element is added
    #[inline]
    pub const fn new() -> Self {
        Self {
            logical_start: 0,
            logical_end: 0,
            contents: Vec::new()
        }
    }
    /// Create a new empty vector with at least the specified capacity preallocated
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            logical_start: 0,
            logical_end: 0,
            contents: Vec::with_capacity(capacity)
        }
    }
    /// Clears the vector, logically removing all values, but not dropping them
    #[inline]
    pub fn clear(&mut self) {
        self.logical_start = 0;
        self.logical_end = 0;
    }
    /// Shortens the vector, keeping the first `len` elements and logically removing the rest
    ///
    /// If `len` is greater or equal to the vector’s current logical length, this has no effect.
    #[inline]
    pub fn truncate(&mut self, len: usize) {
        if len == 0 {
            self.clear()
        } else {
            if len < (self.logical_end - self.logical_start) {
                self.logical_end = self.logical_start + len;
            }
        }
    }
    /// Returns the number of logical elements in the vector, also referred to as its ‘length’
    #[inline]
    pub fn len(&self) -> usize {
        self.logical_end - self.logical_start
    }
    /// Returns `true` if the vector contains no logical elements
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Appends an element to the back of a vector, increasing the logical length by 1,
    /// creating or reinitializing the element with one of the supplied closures
    #[inline]
    pub fn push_with<NewF, ResetF>(&mut self, new_f: NewF, reset_f: ResetF)
        where
        NewF: FnOnce() -> T,
        ResetF: FnOnce(&mut T)
    {
        if self.logical_end < self.contents.len() {
            reset_f(self.contents.get_mut(self.logical_end).unwrap());
        } else {
            self.contents.push(new_f());
        }
        self.logical_end += 1;
    }
    /// Removes the last element from the vector
    ///
    /// Returns a mutable reference to the element that was removed, or `None` if the vector was already empty
    #[inline]
    pub fn pop(&mut self) -> Option<&mut T> {
        if self.logical_end > self.logical_start {
            self.logical_end -= 1;
            let old_idx = self.logical_end;
            if self.logical_end == self.logical_start {
                self.clear();
            }
            self.contents.get_mut(old_idx)
        } else {
            self.clear();
            None
        }
    }
    /// Removes the first element from the vector
    ///
    /// Returns a mutable reference to the element that was removed, or `None` if the vector was already empty
    #[inline]
    pub fn pop_front(&mut self) -> Option<&mut T> {
        if self.logical_end > self.logical_start {
            let old_idx = self.logical_start;
            self.logical_start += 1;
            if self.logical_end == self.logical_start {
                self.clear();
            }
            self.contents.get_mut(old_idx)
        } else {
            self.clear();
            None
        }
    }
}

impl<T> AsMut<[T]> for ReusingQueue<T> {
    fn as_mut(&mut self) -> &mut [T] {
        &mut self.contents[self.logical_start..self.logical_end]
    }
}

impl<T> AsRef<[T]> for ReusingQueue<T> {
    fn as_ref(&self) -> &[T] {
        &self.contents[self.logical_start..self.logical_end]
    }
}

impl<T> core::borrow::Borrow<[T]> for ReusingQueue<T> {
    fn borrow(&self) -> &[T] {
        &self.contents[self.logical_start..self.logical_end]
    }
}

impl<T> core::borrow::BorrowMut<[T]> for ReusingQueue<T> {
    fn borrow_mut(&mut self) -> &mut [T] {
        &mut self.contents[self.logical_start..self.logical_end]
    }
}

impl<T> core::ops::Deref for ReusingQueue<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.contents[self.logical_start..self.logical_end]
    }
}

impl<T> core::ops::DerefMut for ReusingQueue<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.contents[self.logical_start..self.logical_end]
    }
}

impl<T> From<Vec<T>> for ReusingQueue<T> {
    fn from(vec: Vec<T>) -> Self {
        Self {
            logical_start: 0,
            logical_end: vec.len(),
            contents: vec
        }
    }
}

impl<T> From<ReusingQueue<T>> for Vec<T> {
    fn from(mut vec: ReusingQueue<T>) -> Self {
        vec.contents.drain(0..vec.logical_start);
        vec.contents.truncate(vec.logical_end - vec.logical_start);
        vec.contents
    }
}

impl<T, U> FromIterator<U> for ReusingQueue<T> where T: From<U> {
    fn from_iter<I: IntoIterator<Item=U>>(iter: I) -> Self {
        let contents: Vec<T> = iter.into_iter().map(|element| element.into()).collect();
        Self {
            logical_start: 0,
            logical_end: contents.len(),
            contents,
        }
    }
}

impl<T> IntoIterator for ReusingQueue<T> {
    type Item = T;
    type IntoIter = ReusingVecIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        let mut iter = self.contents.into_iter();
        if self.logical_start > 0 {
            iter.nth(self.logical_start - 1);
        }
        iter.take(self.logical_end - self.logical_start)
    }
}

impl<T> PartialEq<Self> for ReusingQueue<T> where T: PartialEq {
    fn eq(&self, other: &Self) -> bool {
        (self as &[T]).eq(other as &[T])
    }
}
impl<T> Eq for ReusingQueue<T> where T: Eq {}

impl<T> PartialEq<[T]> for ReusingQueue<T> where T: PartialEq {
    fn eq(&self, other: &[T]) -> bool {
        (self as &[T]).eq(other)
    }
}

impl<T> PartialEq<Vec<T>> for ReusingQueue<T> where T: PartialEq {
    fn eq(&self, other: &Vec<T>) -> bool {
        (self as &[T]).eq(other)
    }
}

impl<T> PartialEq<ReusingVec<T>> for ReusingQueue<T> where T: PartialEq {
    fn eq(&self, other: &ReusingVec<T>) -> bool {
        (self as &[T]).eq(other as &[T])
    }
}

impl<T: ReusableElement> ReusingQueue<T> {
    /// Appends an empty element to the back of a vector, increasing the logical length by 1, and returns
    /// a mutable reference to the new / re-initialized element
    #[inline]
    pub fn push_mut(&mut self) -> &mut T {
        if self.logical_end < self.contents.len() {
            self.contents.get_mut(self.logical_end).unwrap().reset();
        } else {
            self.contents.push(T::new());
        }
        let element = self.contents.get_mut(self.logical_end).unwrap();
        self.logical_end += 1;
        element
    }
}

#[test]
fn queue_test() {
    let mut queue: ReusingQueue<i32> = (0..10).into_iter().collect();

    assert_eq!(queue.len(), 10);
    assert_eq!(*queue, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    queue.truncate(9);
    assert_eq!(queue.len(), 9);
    assert_eq!(*queue, [0, 1, 2, 3, 4, 5, 6, 7, 8]);
    assert_eq!(queue.pop(), Some(&mut 8));

    queue.pop();
    assert_eq!(queue.pop_front(), Some(&mut 0));
    assert_eq!(queue.len(), 6);
    assert_eq!(*queue, [1, 2, 3, 4, 5, 6]);

    queue.truncate(5);
    assert_eq!(queue.len(), 5);
    assert_eq!(*queue, [1, 2, 3, 4, 5]);

    let vec_1: Vec<i32> = queue.clone().into_iter().collect();
    assert_eq!(*vec_1, *queue);

    let vec_2: Vec<i32> = queue.clone().into();
    assert_eq!(vec_1, vec_2);

    while queue.pop().is_some() {
        queue.pop_front();
    }
    assert_eq!(queue.len(), 0);
    assert_eq!(queue.pop_front(), None);
}