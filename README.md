# RetainMut

Trait that provides `retain_mut` method.

This method is basically the same as `Vec::retain`,
but it gives mutable borrow to the predicate function.

This was probably a historical mistake in Rust library,
that `retain` should do this at the very beginning.
See [rust-lang/rust#25477](https://github.com/rust-lang/rust/issues/25477).

It currently only implements `retain_mut` for `Vec`.
We may implement it for more collection types in the future.
