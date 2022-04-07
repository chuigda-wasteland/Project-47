//! # Pr47(Project-47): A semi-experimental embeddable programming language for Rust
//!
//! ## ⚠⚠⚠ Develop stage note ⚠⚠⚠
//! By this time the author doesn't know which APIs are necessary for user to write low-level,
//! direct FFI bindings, nor does the author know which APIs are necessary for user to tweak the
//! VM runtime. So we are making as many things public as possible. This situation may change in the
//! future so watch your back.

#![allow(
    clippy::unusual_byte_groupings,
    clippy::type_complexity,
    clippy::missing_safety_doc,
    clippy::new_without_default,
    clippy::match_like_matches_macro,
    clippy::not_unsafe_ptr_arg_deref
)]

pub mod builtins;
pub mod data;
pub mod ffi;
pub mod util;
pub mod vm;

#[cfg(feature = "compiler")] pub mod diag;
#[cfg(feature = "compiler")] pub mod parse;
#[cfg(feature = "compiler")] pub mod syntax;
#[cfg(feature = "compiler")] pub mod sema;
#[cfg(feature = "std47")]    pub mod std47;

#[cfg(all(feature = "al31f-builtin-ops", not(feature = "al31f")))]
compile_error!("using `al31f-builtin-ops` without `al31f` is meaningless");

#[cfg(all(feature = "async-avoid-block", not(feature = "async")))]
compile_error!("using `async-avoid-block` without `async` is meaningless");

#[cfg(all(feature = "compiler-pretty-diag", not(feature = "compiler")))]
compile_error!("using `compiler-pretty-diag` without `compiler` is meaningless");

#[cfg(all(feature = "async-astd", feature = "async-tokio"))]
compile_error!("features `async-astd` and `async-tokio` are mutually exclusive");

#[cfg(all(feature = "with-log", feature = "with-tracing"))]
compile_error!("feature `with-log` and `with-tracing` are mutually exclusive");

#[cfg(test)]
#[macro_use] extern crate variant_count;
