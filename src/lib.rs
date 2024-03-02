//! Access guards based on invariant lifetime markers.

mod lifetime;
#[doc(inline)]
pub use lifetime::{Scope, Region, with_region, with_scope};

mod gen;
#[doc(inline)]
pub use gen::{Generative, Gen, GenType, TryGenTuple, DistinctTuple, with_type, with_types};