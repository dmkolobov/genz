//! Uniqueness of types via invariant lifetimes.
//!
//! Provides a `UniqueType` marker which makes it impossible to call the following without resorting to `unsafe` code:
//!
//! ```
//! # use genz::*;
//! fn same_type<'c, T>(t1: UniqueType<'c, T>, t2: UniqueType<'c, T>)
//! {
//!   panic!("this is impossible!")
//! }
//! ```

mod lifetime;
#[doc(inline)]
pub use lifetime::{Scope, Region, with_region, with_scope};

mod storable;
pub use storable::Storable;

mod gen;
#[doc(inline)]
pub use gen::{Gen, UniqueType, TryGenTuple, StaticTuple, with_type, try_with_types, with_types};