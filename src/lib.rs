//! Zero-sized markers for distinct types.
//!
//! A marker `GenType<'c, T>` is __generative__ in the region of code with lifetime `'c` if it is impossible to invoke
//! the following function:
//!
//! ```
//! # use genz::*;
//! fn same_type<'c, T>(t1: GenType<'c, T>, t2: GenType<'c, T>)
//! {
//!   panic!("this is impossible!")
//! }
//! ```

mod lifetime;
#[doc(inline)]
pub use lifetime::{Scope, Region, with_region, with_scope};

mod gen;
#[doc(inline)]
pub use gen::{Storable, Gen, GenType, TryGenTuple, StaticTuple, with_type, try_with_types, with_types};