use std::fmt::Debug;
use std::ptr;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::board::Board;
use crate::evaluation::Evaluation;
use crate::types::chess_move::Move;

#[derive()]
pub struct TranspositionTable {
    inner: slice::CacheAlignedSlice<AtomicU64>,
}

impl TranspositionTable {
    /// Create an uninitialized transposition table
    ///
    /// # Safety
    /// Caller must ensure that the table is initialized before the first store/probe
    pub unsafe fn new_uninitialized(size_mb: usize) -> Self {
        let table_size = 0x100000 * size_mb;
        let num_entries = table_size / std::mem::size_of::<AtomicU64>();

        let inner = unsafe { slice::CacheAlignedSlice::new_uninitialized(num_entries) };

        Self { inner }
    }

    pub fn store(
        &self,
        board: &Board,
        best_move: Move,
        depth: u8,
        mut value: Evaluation,
        value_type: ValueType,
        ply: u8,
    ) {
        let index = board.hash() % self.inner.len() as u64;

        // replacement scheme
        let old_entry = self.inner[index as usize].load(Ordering::Relaxed);

        // SAFETY: we statically asserted that an entry is exactly 8 bytes
        let old_entry: Entry = unsafe { std::mem::transmute(old_entry) };
        if depth < old_entry.depth {
            // don't replace when the new entry is less deeply analyzed as the old entry
            return;
        }

        if value.is_mate() {
            value = value.score_to_tt(ply);
        }

        let entry = Entry {
            hash_key: board.hash() as u16,
            best_move,
            depth,
            value,
            value_type,
        };

        let entry: u64 = unsafe { std::mem::transmute(entry) };

        self.inner[index as usize].store(entry, Ordering::Relaxed);
    }

    pub fn probe(&self, board: &Board, ply: u8) -> Option<Entry> {
        let index = board.hash() % self.inner.len() as u64;

        let entry = self.inner[index as usize].load(Ordering::Relaxed);

        // SAFETY: we statically asserted that an entry is exactly 8 bytes
        let mut entry: Entry = unsafe { std::mem::transmute(entry) };

        if entry.hash_key != board.hash() as u16 {
            return None;
        }

        if entry.value.is_mate() {
            entry.value = entry.value.tt_to_score(ply)
        }

        Some(entry)
    }

    /// # Safety
    ///
    /// Caller must ensure exclusive access to this chunk
    pub unsafe fn clear_chunk(&self, chunk_index: usize, num_chunks: usize) {
        let len = self.inner.len();

        let stride = len / num_chunks;
        let start = stride * chunk_index;
        let end = if chunk_index != num_chunks - 1 {
            (start + stride).min(len)
        } else {
            len
        };

        let start_ptr = unsafe { self.inner.as_ptr().add(start) } as *mut AtomicU64;
        let count = end - start;

        unsafe { ptr::write_bytes(start_ptr, 0, count) };
    }

    pub fn size_mb(&self) -> usize {
        self.inner.len() * std::mem::size_of::<AtomicU64>() / 0x100000
    }
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub hash_key: u16,
    pub best_move: Move,
    pub depth: u8,
    pub value: Evaluation,
    pub value_type: ValueType,
}

// entry has to fit into a u64
const _: () = assert!(std::mem::size_of::<Entry>() == 8);

#[derive(Debug, Clone, PartialEq)]
#[repr(u8)]
pub enum ValueType {
    Exact,
    /// Alpha
    Upperbound,
    /// Beta
    Lowerbound,
}

mod slice {
    use std::alloc::{Layout, alloc, dealloc};
    use std::ops::{Index, IndexMut};
    use std::ptr::NonNull;

    pub struct CacheAlignedSlice<T> {
        ptr: NonNull<T>,
        len: usize,
        layout: Layout,
    }

    impl<T> CacheAlignedSlice<T> {
        pub unsafe fn new_uninitialized(len: usize) -> Self {
            // We do not handle types that need to be dropped
            const { assert!(!std::mem::needs_drop::<T>()) };

            // Only types sizes that divide the cache line size are valid
            // to enable easy chunking
            const { assert!(64 % std::mem::align_of::<T>() == 0) }

            let elem_size = std::mem::size_of::<T>();
            let align = 64.max(std::mem::align_of::<T>());

            let layout = Layout::from_size_align(len * elem_size, align).unwrap();

            // The usage of `alloc` instead of `alloc_zeroed` is intentional.
            // `alloc_zeroed` would return zeroed memory, but pages may be left
            // in an uncommitted state (only lazily mapped by the OS). To avoid
            // first-access latency, we clear the memory manually and use `alloc`
            // since it is faster.
            let ptr = unsafe { alloc(layout) as *mut T };

            let ptr = NonNull::new(ptr)
                .unwrap_or_else(|| panic!("allocation failed for {} bytes", layout.size()));

            Self { ptr, len, layout }
        }

        pub fn len(&self) -> usize {
            self.len
        }

        pub fn as_ptr(&self) -> *const T {
            self.ptr.as_ptr()
        }
    }

    impl<T> Index<usize> for CacheAlignedSlice<T> {
        type Output = T;
        #[inline]
        #[track_caller]
        fn index(&self, idx: usize) -> &Self::Output {
            assert!(
                idx < self.len,
                "index out of bounds: the len is {} but the index is {}",
                self.len,
                idx
            );
            unsafe { &*self.ptr.as_ptr().add(idx) }
        }
    }

    impl<T> IndexMut<usize> for CacheAlignedSlice<T> {
        #[inline]
        #[track_caller]
        fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
            assert!(
                idx < self.len,
                "index out of bounds: the len is {} but the index is {}",
                self.len,
                idx
            );
            unsafe { &mut *self.ptr.as_ptr().add(idx) }
        }
    }

    impl<T> Drop for CacheAlignedSlice<T> {
        fn drop(&mut self) {
            unsafe { dealloc(self.ptr.as_ptr() as *mut u8, self.layout) };
        }
    }

    unsafe impl<T: Sync> Sync for CacheAlignedSlice<T> {}
    unsafe impl<T: Send> Send for CacheAlignedSlice<T> {}
}
