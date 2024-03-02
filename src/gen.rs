//! Access guarding with an invariant lifetime.

use std::{borrow::BorrowMut, marker::PhantomData};
use crate::{lifetime::STATIC_REGION, with_region, Region};

/// The trait of values which may be placed in a generative context.
pub trait Generative: BorrowMut<Self::Guarded<'static>> + From<Self::Guarded<'static>>
{
  /// The value accessible in a generative context with lifetime `'c`.
  type Guarded<'c>;
}

/// Invoke `f` inside a generative context with a marker for a generative type.
#[inline]
pub fn with_type<U, Z>(f: impl for <'c> FnOnce(GenType<'c, U>) -> Z) -> Z 
{
  with_region(|region| f(GenType(region, PhantomData)))
}

/// Invoke `f` inside a generative context with a marker for a generative type.
#[inline]
pub fn with_types<Types: TryGenTuple, Z>(f: impl for <'c> FnOnce(Region<'c>, Types::Tuple<'c>) -> Z) -> Option<Z> 
{
  with_region(|region| Types::try_gen_types(region).map(|types| f(region, types)))
}

/// A context for guarding access to a value via an invariant anonymous lifetime.
#[repr(transparent)]
pub struct Gen<T>(T);

impl<T: Generative> Gen<T>
{
  /// Create a value inside a generative context using `f`.
  #[inline]
  pub fn from_fn(f: impl for <'c> FnOnce(Region<'c>) -> T::Guarded<'c>) -> Self 
  {
    Gen(f(STATIC_REGION).into())
  }

  /// Create a value inside a generative context using `f`.
  #[inline]
  pub fn try_from_fn(f: impl for <'c> FnOnce(Region<'c>) -> Option<T::Guarded<'c>>) -> Option<Self> 
  {
    f(STATIC_REGION).map(|inner| Gen(inner.into()))
  }

  /// Create a value inside a generative context by a type marker to the closure `f`.
  #[inline]
  pub fn from_type<U>(f: impl for <'c> FnOnce(GenType<'c, U>) -> T::Guarded<'c>) -> Self 
  {
    Self::from_fn(|region| f(GenType(region, PhantomData)))
  }

  /// Create a value inside a generative context by passing a tuple of type markers to the closure `f`.
  ///
  /// ```
  /// # use genz::*;
  ///
  /// struct Coll<'c, T>(GenType<'c, T>);
  ///
  /// struct Ctx<'c, A, B>
  /// {
  ///   a: Coll<'c, A>,
  ///   b: Coll<'c, B>,
  /// }
  ///
  /// impl<A, B> Generative for Ctx<'static, A, B>
  /// {
  ///   type Guarded<'c> = Ctx<'c, A, B>;
  /// }
  /// 
  /// let gen = Gen::<Ctx<_, _>>::try_from_types::<(u8, u16)>(|_, (t1, t2)| {
  ///   Ctx {a: Coll(t1), b: Coll(t2)}
  /// }).unwrap();
  /// ```
  #[inline]
  pub fn try_from_types<Types: TryGenTuple>(f: impl for <'c> FnOnce(Region<'c>, Types::Tuple<'c>) -> T::Guarded<'c>) -> Option<Self> 
  {
    Self::try_from_fn(|region| Types::try_gen_types(region).map(|types| f(region, types)))
  }

  /// Invoke `f` with a reference to `self` inside an anonymous generative context.
  #[inline]
  pub fn with_ref<Z>(&self, f: impl for<'c> FnOnce(&T::Guarded<'c>) -> Z) -> Z 
  {
    f(self.0.borrow())
  }

  /// Invoke `f` with a mutable reference to `self` inside an anonymous generative context.
  #[inline] 
  pub fn with_mut<Z>(&mut self, f: impl for<'c> FnOnce(&mut T::Guarded<'c>) -> Z) -> Z 
  {
    f(self.0.borrow_mut())
  }
}

/// A marker for a unique type accessible only in a generative context.
#[repr(transparent)]
pub struct GenType<'c, T>(Region<'c>, PhantomData<T>);

impl<'c, T> From<GenType<'c, T>> for Region<'c>
{
  #[inline]
  fn from(value: GenType<'c, T>) -> Self {
    value.0
  }
}

/// A trait implemented by tuples of distinct types.
pub trait DistinctTuple 
{
  /// Prove that the types in the tuple are distinct. Returns `false` if any type appears more than once in the tuple.
  /// 
  /// ```
  /// # use genz::*;
  ///
  /// assert!(<(u8, u16)>::prove_distinct());
  /// assert!(false == <(u8, u16, u8)>::prove_distinct());
  /// ```
  fn prove_distinct() -> bool;
}

/// A trait for creating a tuples of distinct type markers in a generative context.
pub trait TryGenTuple: DistinctTuple
{
  /// The tuple of types in a generative context with lifetime `'c`.
  type Tuple<'c>;

  /// Generate a series of types accessible only in `region` and return a tuple of type markers.
  fn try_gen_types<'c>(region: Region<'c>) -> Option<Self::Tuple<'c>>;
}

macro_rules! gen_tuple {
    ($($tt:ident),+) => {
      impl<$($tt),+> DistinctTuple for ($($tt,)+)
        where 
          $($tt: 'static),+
      {
        #[inline]
        fn prove_distinct() -> bool 
        {
          let mut ids = [$(std::any::TypeId::of::<$tt>(),)+];
          ids.sort();
          for w in ids.windows(2) {
            if w[0] == w[1] {
              return false;
            }
          }
          true
        }  
      }

      impl<$($tt),+> TryGenTuple for ($($tt,)+)
        where 
          $($tt: 'static),+
      {
        type Tuple<'c> = ($(GenType<'c, $tt>,)+);

        #[inline]
        fn try_gen_types<'c>(region: Region<'c>) -> Option<Self::Tuple<'c>>
        {
          <($($tt,)+)>::prove_distinct().then(|| ($(GenType(region, PhantomData::<$tt>),)+))
        }
      }
    };
} 

gen_tuple!(T0, T1);
gen_tuple!(T0, T1, T2);
gen_tuple!(T0, T1, T2, T3);
gen_tuple!(T0, T1, T2, T3, T4);
gen_tuple!(T0, T1, T2, T3, T4, T5);
gen_tuple!(T0, T1, T2, T3, T4, T5, T6);
gen_tuple!(T0, T1, T2, T3, T4, T5, T6, T7);
gen_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8);
gen_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9);
 