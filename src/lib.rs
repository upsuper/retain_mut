use std::ptr;

/// Trait that provides `retain_mut` method.
pub trait RetainMut<T> {
    /// Retains only the elements specified by the predicate with a mutable borrow.
    ///
    /// In other words, remove all elements `e` such that `f(&mut e)` returns `false`.
    /// This method operates in place and preserves the order of the retained
    /// elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use retain_mut::RetainMut;
    ///
    /// let mut vec = vec![1, 2, 3, 4];
    /// vec.retain_mut(|x| { *x *= 3; *x % 2 == 0 });
    /// assert_eq!(vec, [6, 12]);
    /// ```
    fn retain_mut<F>(&mut self, f: F)
    where
        F: FnMut(&mut T) -> bool;
}

impl<T> RetainMut<T> for Vec<T> {
    // The implementation is based on
    // https://github.com/rust-lang/rust/blob/a67749ae87b1c873ed09fca2a204beff2fe5e7ea/src/liballoc/vec.rs#L804-L829
    fn retain_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        let len = self.len();
        let mut del = 0;
        {
            let v = &mut **self;
            for i in 0..len {
                if !f(&mut v[i]) {
                    del += 1;
                    unsafe {
                        ptr::drop_in_place(&mut v[i]);
                    }
                } else if del > 0 {
                    let src: *const T = &v[i];
                    let dst: *mut T = &mut v[i - del];
                    unsafe {
                        ptr::copy_nonoverlapping(src, dst, 1);
                    }
                }
            }
        }
        unsafe {
            self.set_len(len - del);
        }
    }
}
