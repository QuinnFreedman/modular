use core::ops::{Index, IndexMut};

pub struct RingBuffer<T, const N: usize>
where
    T: Default + Copy,
{
    buf: [T; N],
    head: usize, // points to the oldest element (i.e. the next write position)
}

impl<T, const N: usize> RingBuffer<T, N>
where
    T: Default + Copy,
{
    /// Create a new buffer with all elements initialized to `T::default()`.
    pub fn new() -> Self {
        Self {
            buf: [T::default(); N],
            head: 0,
        }
    }

    /// Push a new element into the buffer, overwriting the oldest.
    pub fn push(&mut self, value: T) {
        self.buf[self.head] = value;
        self.head = (self.head + 1) % N;
    }

    /// Push all elements from a fixed-size array into the buffer.
    pub fn push_all_into<const M: usize>(&mut self, values: [T; M]) {
        for v in values {
            self.push(v);
        }
    }

    /// Push all elements from a fixed-size array into the buffer.
    pub fn push_all<const M: usize>(&mut self, values: &[T; M]) {
        for v in values {
            self.push(v.clone());
        }
    }

    pub fn len() -> usize {
        N
    }

    pub fn as_array(&self) -> [T; N] {
        let mut out = [T::default(); N];
        let mut i = 0;
        while i < N {
            out[i] = self[i];
            i += 1;
        }
        out
    }

    /// Translate a logical index (0 = oldest) into the actual array index.
    fn physical_index(&self, logical: usize) -> usize {
        (self.head + logical) % N
    }
}

impl<T, const N: usize> Index<usize> for RingBuffer<T, N>
where
    T: Default + Copy,
{
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        let pos = (self.head + index) % N;
        &self.buf[pos]
    }
}

impl<T, const N: usize> IndexMut<usize> for RingBuffer<T, N>
where
    T: Default + Copy,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let pos = (self.head + index) % N;
        &mut self.buf[pos]
    }
}

/// Immutable iterator
pub struct RingBufferIter<'a, T, const N: usize>
where
    T: Default + Copy,
{
    rb: &'a RingBuffer<T, N>,
    idx: usize,
}

impl<'a, T, const N: usize> Iterator for RingBufferIter<'a, T, N>
where
    T: Default + Copy,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < N {
            let item = &self.rb[self.idx];
            self.idx += 1;
            Some(item)
        } else {
            None
        }
    }
}

/// Mutable iterator
pub struct RingBufferIterMut<'a, T, const N: usize>
where
    T: Default + Copy,
{
    rb: &'a mut RingBuffer<T, N>,
    idx: usize,
}

impl<'a, T, const N: usize> Iterator for RingBufferIterMut<'a, T, N>
where
    T: Default + Copy,
{
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < N {
            // avoid aliasing by splitting at index
            let pos = self.rb.physical_index(self.idx);
            self.idx += 1;
            // Safety: each element is yielded at most once
            Some(unsafe { &mut *(&mut self.rb.buf[pos] as *mut T) })
        } else {
            None
        }
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a RingBuffer<T, N>
where
    T: Default + Copy,
{
    type Item = &'a T;
    type IntoIter = RingBufferIter<'a, T, N>;

    fn into_iter(self) -> Self::IntoIter {
        RingBufferIter { rb: self, idx: 0 }
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a mut RingBuffer<T, N>
where
    T: Default + Copy,
{
    type Item = &'a mut T;
    type IntoIter = RingBufferIterMut<'a, T, N>;

    fn into_iter(self) -> Self::IntoIter {
        RingBufferIterMut { rb: self, idx: 0 }
    }
}
