//! Anonymous lifetime markers.

use std::marker::PhantomData;

/// A covariant lifetime marker.
///
/// Because `f` is defined for an arbitrary lifetime `'c` and `Z` has a fixed lifetime, values referencing `'c` are 
/// prevented from escaping the closure:
///
/// ```compile_fail
/// # use genz::*;
///
/// struct Hidden<'c>(Scope<'c>);
///
/// let x = with_scope(|s| Hidden(s)); // lifetime may not live long enough
/// ```
///
/// The covariance of `Scope` with respect to its lifetime ensures that a subtyping relation between lifetimes 
/// implies a subtyping relation between `Scope` markers:
///
/// ``` 
/// # use genz::*;
///
/// fn same_scope<'c>(_: Scope<'c>, _: Scope<'c>)
/// {
///   assert!(true);
/// }
///
/// with_scope(|a| with_scope(|b| same_scope(a, b)));
/// ```
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Scope<'c>(PhantomData<&'c ()>);

/// Invoke `f` with an covariant lifetime marker.
/// 
/// Because `f` is defined for an arbitrary lifetime `'c` and `Z` has a fixed lifetime, values referencing `'c` are 
/// prevented from escaping the closure:
///
/// ```compile_fail
/// # use genz::*;
///
/// struct Hidden<'c>(Scope<'c>);
///
/// let x = with_scope(|s| Hidden(s)); // lifetime may not live long enough
/// ```
///
/// The covariance of `Scope` with respect to its lifetime ensures that a subtyping relation between lifetimes 
/// implies a subtyping relation between `Scope` markers:
///
/// ``` 
/// # use genz::*;
///
/// fn same_scope<'c>(_: Scope<'c>, _: Scope<'c>)
/// {
///   assert!(true);
/// }
///
/// with_scope(|a| with_scope(|b| same_scope(a, b)));
/// ```
#[inline]
pub fn with_scope<F, Z>(f: F) -> Z 
  where 
    for<'c> F: FnOnce(Scope<'c>) -> Z 
{
  f(Scope(PhantomData))
}

/// An invariant lifetime marker.
///
/// Region markers are created via the `with_region` function. 
///
/// Because `f` is defined for an arbitrary lifetime `'c` and `Z` has a fixed lifetime, values referencing `'c` are 
/// prevented from escaping the closure:
///
/// ```compile_fail
/// # use genz::*;
///
/// struct Hidden<'c>(Region<'c>);
///
/// let x = with_region(|s| Hidden(s)); // lifetime may not live long enough
/// ```
///
/// The invariance of `Region` with respect to its lifetime ensures that there is no subtyping relation between the 
/// lifetimes of distinct `Region` markers:
///
/// ```compile_fail 
/// # use genz::*;
///
/// fn same_region<'c>(_: Region<'c>, _: Region<'c>)
/// {
/// }
///
/// with_region(|a| with_region(|b| same_region(a, b))); // fails to compile
/// ```
/// 
/// This property is passed through to any type referencing the region:
///
/// ```compile_fail
/// # use genz::*;
/// # use std::marker::PhantomData;
///
/// fn same_region<'c>(_: PhantomData<Region<'c>>, _: PhantomData<Region<'c>>)
/// {}
///
/// fn as_phantom<'c>(_: Region<'c>) -> PhantomData<Region<'c>>
/// { 
///   PhantomData
/// }
///
/// with_region(|a| with_region(|b| same_region(as_phantom(a), as_phantom(b))))
/// ```
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Region<'c>(PhantomData<&'c mut &'c ()>);

/// The static region.
pub const STATIC_REGION: Region<'static> = Region(PhantomData);

/// Invoke `f` with an invariant lifetime marker.
/// 
/// Because `f` is defined for an arbitrary lifetime `'c` and `Z` has a fixed lifetime, values referencing `'c` are 
/// prevented from escaping the closure:
///
/// ```compile_fail
/// # use genz::*;
///
/// struct Hidden<'c>(Region<'c>);
///
/// let x = with_region(|s| Hidden(s)); // lifetime may not live long enough
/// ```
#[inline]
pub fn with_region<F, Z>(f: F) -> Z
  where 
    for<'c> F: FnOnce(Region<'c>) -> Z
{
  f(Region::<'static>(PhantomData))
}