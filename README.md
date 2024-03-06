# genz 
Uniqueness of types via invariant lifetimes.

Provides a non-`Copy` zero-sized type-marker called `UniqueType`, which makes it impossible to call the following
without resorting to `unsafe` code:

```rust
# use genz::*;
fn same_type<'c, T>(t1: UniqueType<'c, T>, t2: UniqueType<'c, T>)
{
  panic!("this is impossible!")
}
```