//! Access guarding with an invariant lifetime.

use std::{borrow::BorrowMut, marker::PhantomData};
use crate::{lifetime::STATIC_REGION, with_region, Region, Storable};

/// Invoke `f` with a type marker which is unique for an invariant lifetime.
#[inline]
pub fn with_type<U, Z>(f: impl for <'c> FnOnce(UniqueType<'c, U>) -> Z) -> Z 
{
  with_region(|region| f(UniqueType(region, PhantomData)))
}

/// Attempt to invoke `f` with an invariant lifetime marker and a tuple of type markers that are unique for that lifetime.
///
/// If any types in the tuple are duplicates, `None` is returned.
#[inline]
pub fn try_with_types<Types: TryGenTuple, Z>(f: impl for <'c> FnOnce(Region<'c>, Types::Tuple<'c>) -> Z) -> Option<Z> 
{
  with_region(|region| Types::try_gen_tuple(region).map(|types| f(region, types)))
}

/// Like `try_with_types`, but it `unwrap`s for you.
#[inline]
pub fn with_types<Types: TryGenTuple, Z>(f: impl for <'c> FnOnce(Region<'c>, Types::Tuple<'c>) -> Z) -> Z
{
  try_with_types(f).unwrap()
}

/// A structure for storing values containing unique types.
#[repr(transparent)]
pub struct Gen<T>(T);

impl<Z: Storable> Gen<Z>
{
  #[inline]
  fn from_fn(f: impl for <'c> FnOnce(Region<'c>) -> Z::Generative<'c>) -> Self 
  {
    Gen(f(STATIC_REGION).into())
  }

  #[inline]
  fn try_from_fn(f: impl for <'c> FnOnce(Region<'c>) -> Option<Z::Generative<'c>>) -> Option<Self> 
  {
    f(STATIC_REGION).map(|inner| Gen(inner.into()))
  }

  /// Created a stored value by invoking `f` with a type marker which is unique for an invariant lifetime.
  #[inline]
  pub fn from_type<U>(f: impl for <'c> FnOnce(UniqueType<'c, U>) -> Z::Generative<'c>) -> Self 
  {
    Self::from_fn(|region| f(UniqueType(region, PhantomData)))
  }

  /// Attempt to create a stored value by invoking `f` with an invariant lifetime marker and a tuple of type markers that are unique for that lifetime.
  ///
  /// If any types in the tuple are duplicates, `None` is returned.
  /// ```
  /// # use genz::*;
  /// let gen = Gen::<(UniqueType<u8>, UniqueType<u16>)>::try_from_types::<(u8, u16)>(|_, (t1, t2)| {
  ///   (t1, t2)
  /// }).unwrap();
  ///
  /// // `None` is returned because `u8` is repeated in the tuple
  /// assert!(Gen::<(UniqueType<u8>, UniqueType<u8>)>::try_from_types::<(u8, u8)>(|_, (t1, t2)| {
  ///   (t1, t2)
  /// }).is_none())
  /// ```
  #[inline]
  pub fn try_from_types<Types: TryGenTuple>(f: impl for <'c> FnOnce(Region<'c>, Types::Tuple<'c>) -> Z::Generative<'c>) -> Option<Self> 
  {
    Self::try_from_fn(|region| Types::try_gen_tuple(region).map(|types| f(region, types)))
  }

  /// Like `try_from_types`, but it `unwrap`s for you.
  #[inline]
  pub fn from_types<Types: TryGenTuple>(f: impl for <'c> FnOnce(Region<'c>, Types::Tuple<'c>) -> Z::Generative<'c>) -> Self 
  {
    Self::try_from_types(f).unwrap()
  }

  /// Invoke `f` with a reference to the value.
  #[inline]
  pub fn with_ref<R>(&self, f: impl for<'c> FnOnce(&Z::Generative<'c>) -> R) -> R 
  {
    f(self.0.borrow())
  }

  /// Invoke `f` with a mutable reference to the value.
  #[inline] 
  pub fn with_mut<R>(&mut self, f: impl for<'c> FnOnce(&mut Z::Generative<'c>) -> R) -> R 
  {
    f(self.0.borrow_mut())
  }
}

/// A marker for a type which is guaranteed to be unique within some region of code.
///
/// When we have a `UniqueType<'c, T>`, then the type `T` is guaranteed to be unique for lifetime `'c`. 
///
/// More precisely, it is impossible to call the following without resorting to `unsafe` code:
///
/// ```
/// # use genz::*;
/// fn same_type<'c, T>(t1: UniqueType<'c, T>, t2: UniqueType<'c, T>)
/// {
///   panic!("this is impossible!")
/// }
/// ```
/// 
/// The following fails because `UniqueType` is not `Copy`: 
/// 
/// ```compile_fail
/// # use genz::*;
/// # fn same_type<'c, T>(t1: UniqueType<'c, T>, t2: UniqueType<'c, T>) {}
/// with_type::<u8, _>(|t1: UniqueType<'_, u8>| {
/// 	same_type(t1, t1);
/// });
/// ```
/// 
/// whereas the following fails because because a `UniqueType` internally contains a `Region` marker, which is invariant
/// with respect to its lifetime:
/// 
/// ```compile_fail
/// # use genz::*;
/// # fn same_type<'c, T>(t1: UniqueType<'c, T>, t2: UniqueType<'c, T>) {}
/// with_type::<u8, _>(|t1: UniqueType<'_, u8>| {
/// 	with_type::<u8, _>(|t2: UniqueType<'_, u8>| {
/// 		same_type(t1, t2); // fails because `t1` and `t2` are tagged with different lifetimes
/// 	});	
/// });
/// ```
///
/// In fact, using `with_type`, we can't even call `different_type`: 
/// 
/// ```compile_fail
/// # use genz::*;
/// fn different_type<'c, T, U>(t1: UniqueType<'c, T>, t2: UniqueType<'c, U>) 
/// {
/// 
/// }
/// 
/// with_type::<u8, _>(|t1: UniqueType<'_, u8>| {
/// 	with_type::<u16, _>(|t2: UniqueType<'_, u16>| {
///         // fails to compile because `t1` and `t2` have different lifetimes
/// 		different_type(t1, t2);
/// 	});	
/// });
/// ```
/// 
/// For this, we'll need the function `try_with_types`, which creates a tuple of unique type markers in 
/// the same region: 
/// 
/// ``` 
/// use genz::{UniqueType, try_with_types};
/// # fn different_type<'c, T, U>(t1: UniqueType<'c, T>, t2: UniqueType<'c, U>) {}
/// 
/// let result = try_with_types::<(u8, u16), _>(|_, (t1, t2): (UniqueType<'_, u8>, UniqueType<'_, u16>)| {
/// 	different_type(t1, t2);
/// });
/// 
/// assert_eq!(Some(()), result);
/// ```
/// 
/// Notice that `try_with_types` returns an `Option`. This is because the function needs to prove that its input types 
/// are distinct, but it cannot do so at compile time(pending stabilization negative traits and auto impls). 
/// If we annotate the function call with a tuple containing duplicates, it will return `None`:
/// 
/// ``` 
/// # use genz::*;
/// 
/// assert_eq!(None, try_with_types::<(u8, u16, u8), _>(|_, _| panic!("should not happen")));
/// ```
#[repr(transparent)]
pub struct UniqueType<'c, T>(Region<'c>, PhantomData<T>);

impl<'c, T> From<UniqueType<'c, T>> for Region<'c>
{
  #[inline]
  fn from(value: UniqueType<'c, T>) -> Self {
    value.0
  }
}

/// A trait implemented by tuples of static types.
pub trait StaticTuple 
{
  /// Returns `false` if any type appears more than once in the tuple, and `true` if all types are distinct.
  /// 
  /// ```
  /// # use genz::*;
  ///
  /// assert!(<(u8, u16)>::distinct());
  /// assert!(false == <(u8, u16, u8)>::distinct());
  /// ```
  fn distinct() -> bool;
}

/// A trait for creating a tuples of unique type markers.
pub trait TryGenTuple: StaticTuple
{
  /// A tuple of type markers which are unique for the lifetime `'c`.
  type Tuple<'c>;

  /// Returns a tuple of type markers which are unique for lifetime `'c` if every type in `Self` is distinct.
  fn try_gen_tuple<'c>(region: Region<'c>) -> Option<Self::Tuple<'c>>;
}

macro_rules! gen_tuple {
    ($($tt:ident),+) => {
      impl<$($tt),+> StaticTuple for ($($tt,)+)
        where 
          $($tt: 'static),+
      {
        #[inline]
        fn distinct() -> bool 
        {
          let ids = [$(std::any::TypeId::of::<$tt>(),)+];
          for i in 0 .. ids.len() {
            for j in i + 1 .. ids.len() {
              if ids[i] == ids[j] {
                return false;
              }
            }
          }
          true
        }  
      }

      impl<$($tt),+> Storable for ($($tt,)+)
        where 
          $($tt: Storable,)+
          ($($tt,)+): From<($($tt::Generative<'static>,)+)>,
          ($($tt,)+): BorrowMut<($($tt::Generative<'static>,)+)>
      {
        type Generative<'c> = ($($tt::Generative<'c>,)+);
      }

      impl<$($tt),+> TryGenTuple for ($($tt,)+)
        where 
          $($tt: 'static),+
      {
        type Tuple<'c> = ($(UniqueType<'c, $tt>,)+);

        #[inline]
        fn try_gen_tuple<'c>(region: Region<'c>) -> Option<Self::Tuple<'c>>
        {
          <($($tt,)+)>::distinct().then(|| ($(UniqueType(region, PhantomData::<$tt>),)+))
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
 