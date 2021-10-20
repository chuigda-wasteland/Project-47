//! ## `type_assert.rs`: defines type assertions used by proc-macro generated codes.
//!
//! The purpose of module `type_assert` is similar to `static_assert`s in C++, which makes the
//! diagnostics information shorter and clearer. If the desired property of one type is not
//! satisfied, compilation will fail then, with error information indicated.

use xjbutil::void::Void;

use crate::data::traits::StaticBase;

/// Contains traits used by type assertions.
///
/// **Warning**: These traits are really internal and made public due to Rust restrictions.
/// Implementing any of these traits will *NOT* magically make your own `Result` or `Option` or so
/// work, but will create bugs that are really hard to troubleshoot. Simply don't do that.
pub mod helper_traits {
    use xjbutil::void::Void;

    /// Trait used to assert that one type is cloneable.
    pub unsafe trait AssertClone<T> {}

    /// Trait used to assert that one type is a reference.
    pub unsafe trait AssertRef<T> {}

    /// Trait used to assert that one type is an immutable reference
    pub unsafe trait AssertConstRef<T> {}

    /// Trait used to assert that one type is a mutable reference.
    pub unsafe trait AssertMutRef<T> {}

    /// Trait used to assert that one type is an exception-convertible `Result` type.
    pub unsafe trait AssertResult<T> {}

    /// Trait used to assert that one type is a null-convertible `Option` type.
    pub unsafe trait AssertOption<T> {}

    unsafe impl<T: Clone> AssertClone<T> for Void {}

    unsafe impl<T> AssertRef<&T> for Void {}
    unsafe impl<T> AssertRef<&mut T> for Void {}

    unsafe impl<T> AssertConstRef<&T> for Void {}

    unsafe impl<T> AssertMutRef<&mut T> for Void {}

    unsafe impl<T, E: 'static> AssertResult<core::result::Result<T, E>> for Void {}
    unsafe impl<T> AssertOption<core::option::Option<T>> for Void {}
}

use crate::util::type_assert::helper_traits::{
    AssertClone,
    AssertConstRef,
    AssertMutRef,
    AssertOption,
    AssertRef,
    AssertResult
};

/// Assert that the type parameter `T` is cloneable.
///
/// ```
/// # fn main() {
/// // Succeeds because std::string::String is cloneable
/// pr47::util::type_assert::assert_clone::<String>();
/// # }
/// ```
///
/// ```compile_fail(E0277)
/// # fn main() {
/// // Fails because std::thread::JoinHandle is not cloneable
/// pr47::util::type_assert::assert_clone::<std::thread::JoinHandle<()>>();
/// # }
/// ```
#[inline(always)] pub const fn assert_clone<T>() where Void: AssertClone<T> {}

/// Assert that the type parameter `T` is a reference.
///
/// ```
/// # fn main() {
/// // Succeeds because &T is a reference type
/// pr47::util::type_assert::assert_ref::<&i64>();
/// // Also succeeds, because &mut T is also a reference type
/// pr47::util::type_assert::assert_ref::<&mut i64>();
/// # }
/// ```
///
/// ```compile_fail(E0277)
/// # fn main() {
/// // Fails because std::string::String is not a reference type
/// pr47::util::type_assert::assert_ref::<String>();
/// # }
/// ```
#[inline(always)] pub const fn assert_ref<T>() where Void: AssertRef<T> {}

/// Assert that the type parameter `T` is an immutable reference.
///
/// ```
/// # fn main() {
/// // Succeeds because &T is an immutable reference type
/// pr47::util::type_assert::assert_const_ref::<&i64>();
/// # }
/// ```
///
/// ```compile_fail(E0277)
/// # fn main() {
/// // Fails because std::string::String is not a reference type
/// pr47::util::type_assert::assert_const_ref::<String>();
/// # }
/// ```
///
/// ```compile_fail(E0277)
/// # fn main() {
/// // Fails because &mut T is an mutable reference, not immmutable
/// pr47::util::type_assert::assert_const_ref::<&mut i64>();
/// # }
/// ```
#[inline(always)] pub const fn assert_const_ref<T>() where Void: AssertConstRef<T>{}

/// Assert that the type parameter `T` is a mutable reference.
///
/// ```
/// # fn main() {
/// // Succeeds because &mut T is a mutable reference type
/// pr47::util::type_assert::assert_mut_ref::<&mut i64>();
/// # }
/// ```
///
/// ```compile_fail(E0277)
/// # fn main() {
/// // Fails because &T is not a mutable reference type
/// pr47::util::type_assert::assert_mut_ref::<&i64>();
/// # }
/// ```
///
/// ```compile_fail(E0277)
/// # fn main() {
/// // Fails because std::string::String is not a mutable reference type
/// pr47::util::type_assert::assert_mut_ref::<String>();
/// # }
/// ```
#[inline(always)] pub const fn assert_mut_ref<T>() where Void: AssertMutRef<T> {}

/// Assert that the type parameter `T` is an exception-convertible `Result` type.
///
/// ```
/// # fn main() {
/// // Succeeds because std::result::Result<(), Box<dyn std::error::Error>> is convertible
/// pr47::util::type_assert::assert_result::<Result<(), Box<dyn std::error::Error>>>();
/// # }
/// ```
///
/// ```compile_fail(E0277)
/// # fn main() {
/// // Fails because std::fs::OpenOptions is not Result type
/// pr47::util::type_assert::assert_result::<std::fs::OpenOptions>();
/// # }
/// ```
///
/// ```compile_fail(E0277)
/// # fn main() {
/// struct Result<T, E>(std::marker::PhantomData<(T, E)>);
/// // Fails because this Result is not std::result::Result
/// pr47::util::type_assert::assert_result::<Result<(), Box<dyn std::error::Error>>>();
/// # }
/// ```
///
/// ```compile_fail(E0277)
/// fn foo<'a>() {
///     // Fails because the error type used is not 'static
///     pr47::util::type_assert::assert_result::<Result<(), &'a i64>>();
/// }
/// ```
#[inline(always)] pub const fn assert_result<T>() where Void: AssertResult<T> {}

/// Assert that the type parameter `T` is an null-convertible `Option` type.
///
/// ```
/// # fn main() {
/// // Succeeds because std::option::Option<std::collections::Vec<u8>> is convertible
/// pr47::util::type_assert::assert_option::<Option<Vec<u8>>>();
/// # }
/// ```
///
/// ```compile_fail(E0277)
/// # fn main() {
/// // Fails because std::fs::File is not Option type
/// pr47::util::type_assert::assert_option::<std::fs::File>();
/// # }
/// ```
///
/// ```compile_fail(E0277)
/// # fn main() {
/// struct Option<T>(std::marker::PhantomData<T>);
/// // Fails because this Option is not std::option::Option
/// pr47::util::type_assert::assert_option::<Option<Vec<u8>>>();
/// # }
/// ```
#[inline(always)] pub const fn assert_option<T>() where Void: AssertOption<T> {}

/// Assert that the type parameter `T` satisfies `StaticBase` requirements.
///
/// ```
/// # fn main() {
/// // Succeeds because std::string::String satisfies StaticBase
/// pr47::util::type_assert::assert_static_base::<String>();
/// # }
/// ```
///
/// ```compile_fail(E0277)
/// # fn main() {
/// // Fails because i64 does not satisfy StaticBase
/// pr47::util::type_assert::assert_static_base::<i64>();
/// # }
/// ```
#[inline(always)] pub const fn assert_static_base<T: 'static>() where Void: StaticBase<T> {}
