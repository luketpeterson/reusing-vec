// Overrides for `docs.rs` links in the README. This first definition takes precedence.

#![cfg_attr(feature = "std", doc = "[Vec]: std::vec::Vec")]

// Normal link to crate items:
//! [`ReusingVec`]: ReusingVec
//! [`ReusingQueue`]: ReusingQueue
//! [`pop_front`]: ReusingQueue::pop_front

#![doc = include_str!("../README.md")]

#![no_std]

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::*;

mod queue;
pub use queue::ReusingQueue;

/// A wrapper around [`Vec`] that supports reusing contained elements without dropping them
///
/// NOTE: Many interfaces are missing or different because the recommended usage pattern is
/// to leave elements in the vector, therefore interfaces like `pop` and `remove` are not
/// included.  They could be, but that might (partially) defeat the purpose of this crate.
///
/// However, if there is an interface you would like, please raise an issue.
#[derive(Clone, Default)]
pub struct ReusingVec<T> {
    logical_len: usize,
    contents: Vec<T>
}

impl<T> core::fmt::Debug for ReusingVec<T> where T: core::fmt::Debug {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "{:?}", self as &[_])
    }
}

impl<T> ReusingVec<T> {
    /// Create a new empty vector, does not allocate until the first element is added
    #[inline]
    pub const fn new() -> Self {
        Self {
            logical_len: 0,
            contents: Vec::new()
        }
    }
    /// Create a new empty vector with at least the specified capacity preallocated
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            logical_len: 0,
            contents: Vec::with_capacity(capacity)
        }
    }
    /// Clears the vector, logically removing all values, but not dropping them
    #[inline]
    pub fn clear(&mut self) {
        self.logical_len = 0;
    }
    /// Shortens the vector, keeping the first `len` elements and logically removing the rest
    ///
    /// If `len` is greater or equal to the vector’s current logical length, this has no effect.
    #[inline]
    pub fn truncate(&mut self, len: usize) {
        if len < self.logical_len {
            self.logical_len = len;
        }
    }
    /// Returns the number of logical elements in the vector, also referred to as its ‘length’
    #[inline]
    pub fn len(&self) -> usize {
        self.logical_len
    }
    /// Returns `true` if the vector contains no logical elements
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.logical_len == 0
    }
    /// Appends the provided element to the back of a vector, increasing the logical length by 1
    ///
    /// NOTE: This operation may cause an inactive `T` to be dropped, as it is overwritten by the
    /// new element provided, so [`push_with`](Self::push_with) or [`push_mut`](Self::push_mut)
    /// is the usual way to get the full benefit from this crate.
    #[inline]
    pub fn push_val(&mut self, val: T) {
        if self.logical_len < self.contents.len() {
            *self.contents.get_mut(self.logical_len).unwrap() = val;
        } else {
            self.contents.push(val);
        }
        self.logical_len += 1;
    }
    /// Appends an element to the back of a vector, increasing the logical length by 1,
    /// creating or reinitializing the element with one of the supplied closures
    #[inline]
    pub fn push_with<NewF, ResetF>(&mut self, new_f: NewF, reset_f: ResetF)
        where
        NewF: FnOnce() -> T,
        ResetF: FnOnce(&mut T)
    {
        if self.logical_len < self.contents.len() {
            reset_f(self.contents.get_mut(self.logical_len).unwrap());
        } else {
            self.contents.push(new_f());
        }
        self.logical_len += 1;
    }
    /// Removes the last element from the vector
    ///
    /// Returns a mutable reference to the element that was removed, or `None` if the vector was already empty
    #[inline]
    pub fn pop(&mut self) -> Option<&mut T> {
        if self.logical_len > 0 {
            self.logical_len -= 1;
            self.contents.get_mut(self.logical_len)
        } else {
            None
        }
    }
}

impl<T> AsMut<[T]> for ReusingVec<T> {
    fn as_mut(&mut self) -> &mut [T] {
        &mut self.contents[0..self.logical_len]
    }
}

impl<T> AsRef<[T]> for ReusingVec<T> {
    fn as_ref(&self) -> &[T] {
        &self.contents[0..self.logical_len]
    }
}

impl<T> core::borrow::Borrow<[T]> for ReusingVec<T> {
    fn borrow(&self) -> &[T] {
        &self.contents[0..self.logical_len]
    }
}

impl<T> core::borrow::BorrowMut<[T]> for ReusingVec<T> {
    fn borrow_mut(&mut self) -> &mut [T] {
        &mut self.contents[0..self.logical_len]
    }
}

impl<T> core::ops::Deref for ReusingVec<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.contents[0..self.logical_len]
    }
}

impl<T> core::ops::DerefMut for ReusingVec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.contents[0..self.logical_len]
    }
}

//NOTE: The extend interface defeats the point of this structure, so I'd rather not support it
// impl<T: ReusableElement> Extend<T::ArgT> for ReusingVec<T> {
//     fn extend<I>(&mut self, iter: I) where I: IntoIterator<Item = T::ArgT> {
//         for element_args in iter {
//             self.push(element_args);
//         }
//     }
// }

impl<T> From<Vec<T>> for ReusingVec<T> {
    fn from(vec: Vec<T>) -> Self {
        Self {
            logical_len: vec.len(),
            contents: vec
        }
    }
}

impl<T> From<ReusingVec<T>> for Vec<T> {
    fn from(mut vec: ReusingVec<T>) -> Self {
        vec.contents.truncate(vec.logical_len);
        vec.contents
    }
}

impl<T, U> FromIterator<U> for ReusingVec<T> where T: From<U> {
    fn from_iter<I: IntoIterator<Item=U>>(iter: I) -> Self {
        let contents: Vec<T> = iter.into_iter().map(|element| element.into()).collect();
        Self {
            logical_len: contents.len(),
            contents,
        }
    }
}

impl<T> IntoIterator for ReusingVec<T> {
    type Item = T;
    type IntoIter = ReusingVecIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        self.contents.into_iter().take(self.logical_len)
    }
}

/// An [`Iterator`] created from a [`ReusingVec`]
pub type ReusingVecIter<T> = core::iter::Take<alloc::vec::IntoIter<T>>;

impl<T> PartialEq<Self> for ReusingVec<T> where T: PartialEq {
    fn eq(&self, other: &Self) -> bool {
        (self as &[T]).eq(other as &[T])
    }
}
impl<T> Eq for ReusingVec<T> where T: Eq {}

impl<T> PartialEq<[T]> for ReusingVec<T> where T: PartialEq {
    fn eq(&self, other: &[T]) -> bool {
        (self as &[T]).eq(other)
    }
}

impl<T> PartialEq<Vec<T>> for ReusingVec<T> where T: PartialEq {
    fn eq(&self, other: &Vec<T>) -> bool {
        (self as &[T]).eq(other)
    }
}

impl<T: ReusableElement> ReusingVec<T> {
    /// Appends an empty element to the back of a vector, increasing the logical length by 1, and returns
    /// a mutable reference to the new / re-initialized element
    #[inline]
    pub fn push_mut(&mut self) -> &mut T {
        if self.logical_len < self.contents.len() {
            self.contents.get_mut(self.logical_len).unwrap().reset();
        } else {
            self.contents.push(T::new());
        }
        let element = self.contents.get_mut(self.logical_len).unwrap();
        self.logical_len += 1;
        element
    }
}

/// Implemented on element types to provide a unified interface for creating a new element and
/// reinitializing an existing element
pub trait ReusableElement {
    fn reset(&mut self);
    fn new() -> Self;
}

impl<T> ReusableElement for Option<T> {
    fn reset(&mut self) {
        *self = None
    }
    fn new() -> Self {
        None
    }
}

impl<T> ReusableElement for Vec<T> {
    fn reset(&mut self) {
        self.clear()
    }
    fn new() -> Self {
        Self::new()
    }
}

impl ReusableElement for String {
    fn reset(&mut self) {
        self.clear()
    }
    fn new() -> Self {
        Self::new()
    }
}

impl<T: Ord> ReusableElement for BinaryHeap<T> {
    fn reset(&mut self) {
        self.clear()
    }
    fn new() -> Self {
        Self::new()
    }
}

impl<K, V> ReusableElement for BTreeMap<K, V> {
    fn reset(&mut self) {
        self.clear()
    }
    fn new() -> Self {
        Self::new()
    }
}

impl<T> ReusableElement for BTreeSet<T> {
    fn reset(&mut self) {
        self.clear()
    }
    fn new() -> Self {
        Self::new()
    }
}

impl<T> ReusableElement for LinkedList<T> {
    fn reset(&mut self) {
        self.clear()
    }
    fn new() -> Self {
        Self::new()
    }
}

impl<T> ReusableElement for VecDeque<T> {
    fn reset(&mut self) {
        self.clear()
    }
    fn new() -> Self {
        Self::new()
    }
}

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
impl<K, V> ReusableElement for std::collections::HashMap<K, V> {
    fn reset(&mut self) {
        self.clear()
    }
    fn new() -> Self {
        Self::new()
    }
}

#[cfg(feature = "std")]
impl<T> ReusableElement for std::collections::HashSet<T> {
    fn reset(&mut self) {
        self.clear()
    }
    fn new() -> Self {
        Self::new()
    }
}

#[cfg(feature = "smallvec")]
impl<A: smallvec::Array> ReusableElement for smallvec::SmallVec<A> {
    fn reset(&mut self) {
        self.clear()
    }
    fn new() -> Self {
        Self::new()
    }
}
