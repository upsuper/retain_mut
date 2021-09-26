//! This crate provides trait `RetainMut` which
//! provides `retain_mut` method for `Vec` and `VecDeque`.
//!
//! `retain_mut` is basically the same as `retain` except that
//! it gives mutable reference of items to the predicate function.
//!
//! Since there is no reason `retain` couldn't have been designed this way,
//! this crate basically just copies the code from std with minor changes
//! to hand out mutable reference.
//! The code these impls are based on can be found in code comments of this crate.
//!
//! This was probably a historical mistake in Rust library,
//! that `retain` should do this at the very beginning.
//! See [rust-lang/rust#25477](https://github.com/rust-lang/rust/issues/25477).
//!
//! ## Compatibility
//!
//! Use `features = ["std"]` for compatibility with Rust version earlier than 1.36,
//! as `no_std` requires `alloc` crate to be stable.
//!
//! ## Examples
//!
//! ### `Vec`
//!
//! ```
//! # use retain_mut::RetainMut;
//! let mut vec = vec![1, 2, 3, 4];
//! vec.retain_mut(|x| { *x *= 3; *x % 2 == 0 });
//! assert_eq!(vec, [6, 12]);
//! ```
//!
//! ### `VecDeque`
//!
//! ```
//! # use retain_mut::RetainMut;
//! # use std::collections::VecDeque;
//! let mut deque = VecDeque::from(vec![1, 2, 3, 4]);
//! deque.retain_mut(|x| { *x *= 3; *x % 2 == 0 });
//! assert_eq!(deque, [6, 12]);
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::collections::vec_deque::VecDeque;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(not(feature = "std"))]
use core::ptr;

#[cfg(feature = "std")]
use std::collections::VecDeque;
#[cfg(feature = "std")]
use std::ptr;

/// Trait that provides `retain_mut` method.
pub trait RetainMut<T> {
    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all elements `e` such that `f(&e)` returns `false`.
    /// This method operates in place, visiting each element exactly once in the
    /// original order, and preserves the order of the retained elements.
    fn retain_mut<F>(&mut self, f: F)
    where
        F: FnMut(&mut T) -> bool;
}

impl<T> RetainMut<T> for Vec<T> {
    // The implementation is based on
    // https://github.com/rust-lang/rust/blob/1d99508b52499c9efd213738e71927458c1d394e/library/alloc/src/vec/mod.rs#L1435-L1508
    fn retain_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        let original_len = self.len();
        // Avoid double drop if the drop guard is not executed,
        // since we may make some holes during the process.
        unsafe { self.set_len(0) };

        // Vec: [Kept, Kept, Hole, Hole, Hole, Hole, Unchecked, Unchecked]
        //      |<-              processed len   ->| ^- next to check
        //                  |<-  deleted cnt     ->|
        //      |<-              original_len                          ->|
        // Kept: Elements which predicate returns true on.
        // Hole: Moved or dropped element slot.
        // Unchecked: Unchecked valid elements.
        //
        // This drop guard will be invoked when predicate or `drop` of element panicked.
        // It shifts unchecked elements to cover holes and `set_len` to the correct length.
        // In cases when predicate and `drop` never panick, it will be optimized out.
        struct BackshiftOnDrop<'a, T> {
            v: &'a mut Vec<T>,
            processed_len: usize,
            deleted_cnt: usize,
            original_len: usize,
        }

        impl<T> Drop for BackshiftOnDrop<'_, T> {
            fn drop(&mut self) {
                if self.deleted_cnt > 0 {
                    // SAFETY: Trailing unchecked items must be valid since we never touch them.
                    unsafe {
                        ptr::copy(
                            self.v.as_ptr().add(self.processed_len),
                            self.v
                                .as_mut_ptr()
                                .add(self.processed_len - self.deleted_cnt),
                            self.original_len - self.processed_len,
                        );
                    }
                }
                // SAFETY: After filling holes, all items are in contiguous memory.
                unsafe {
                    self.v.set_len(self.original_len - self.deleted_cnt);
                }
            }
        }

        let mut g = BackshiftOnDrop {
            v: self,
            processed_len: 0,
            deleted_cnt: 0,
            original_len,
        };

        while g.processed_len < original_len {
            // SAFETY: Unchecked element must be valid.
            let cur = unsafe { &mut *g.v.as_mut_ptr().add(g.processed_len) };
            if !f(cur) {
                // Advance early to avoid double drop if `drop_in_place` panicked.
                g.processed_len += 1;
                g.deleted_cnt += 1;
                // SAFETY: We never touch this element again after dropped.
                unsafe { ptr::drop_in_place(cur) };
                // We already advanced the counter.
                continue;
            }
            if g.deleted_cnt > 0 {
                // SAFETY: `deleted_cnt` > 0, so the hole slot must not overlap with current element.
                // We use copy for move, and never touch this element again.
                unsafe {
                    let hole_slot = g.v.as_mut_ptr().add(g.processed_len - g.deleted_cnt);
                    ptr::copy_nonoverlapping(cur, hole_slot, 1);
                }
            }
            g.processed_len += 1;
        }

        // All item are processed. This can be optimized to `set_len` by LLVM.
        drop(g);
    }
}

impl<T> RetainMut<T> for VecDeque<T> {
    // The implementation is based on
    // https://github.com/rust-lang/rust/blob/0eb878d2aa6e3a1cb315f3f328681b26bb4bffdb/src/liballoc/collections/vec_deque.rs#L1978-L1995
    fn retain_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        let len = self.len();
        let mut del = 0;
        for i in 0..len {
            if !f(&mut self[i]) {
                del += 1;
            } else if del > 0 {
                self.swap(i - del, i);
            }
        }
        if del > 0 {
            self.truncate(len - del);
        }
    }
}
