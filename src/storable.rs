use std::borrow::BorrowMut;

use crate::{Gen, Region, UniqueType};

/// The trait of values containing unique types which may be stored.
///
/// Implementing this trait for your own types is easy:
///
/// ```
/// # use genz::*;
/// pub struct MyStruct<'c, T> {
///   ty: UniqueType<'c, T>,
///   name: &'static str 
/// }
///
/// impl<T> Storable for MyStruct<'static, T> 
/// {
///   type Generative<'c> = MyStruct<'c, T>;
/// }
///
/// let mut x = Gen::<MyStruct<'static, _>>::from_type::<u8>(|ty| MyStruct { ty, name: "u8" });
/// assert_eq!("u8", x.with_ref(|s| s.name));
/// x.with_mut(|s| s.name = "foo");
/// assert_eq!("foo", x.with_ref(|s| s.name));
/// ```
pub trait Storable: BorrowMut<Self::Generative<'static>> + From<Self::Generative<'static>> + Into<Self::Generative<'static>>
{
  /// A value containing types which are unique for lifetime '`c`.
  type Generative<'c>;
}

impl Storable for Region<'static> {
  type Generative<'c> = Region<'c>;
}
  
impl<T> Storable for UniqueType<'static, T> {
  type Generative<'c> = UniqueType<'c, T>;
}

impl<T> Storable for Gen<T> {
  type Generative<'c> = Gen<T>;
}