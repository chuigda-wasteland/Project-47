//! # Pr47(Project-47): A semi-experimental embeddable programming language for Rust
//!
//! ## ⚠⚠⚠ Develop stage note ⚠⚠⚠
//! By this time the author doesn't know which APIs are necessary for user to write low-level,
//! direct FFI bindings, nor does the author know which APIs are necessary for user to tweak the
//! VM runtime. So we are making as many things public as possible. This situation may change in the
//! future so watch your back.

pub mod collections;
pub mod data;
pub mod diag;
pub mod ffi;
pub mod parse;
pub mod sema;
pub mod syntax;
pub mod util;
pub mod vm;

#[cfg(feature = "std47")] pub mod std47;

#[cfg(all(feature = "compiler-pretty-diag", not(feature = "compiler")))]
compile_error!("using `compiler-pretty-diag` without `compiler` is meaningless");

#[cfg(all(feature = "async-astd", feature = "async-tokio"))]
compile_error!("features `async-astd` and `async-tokio` are mutually exclusive");

#[cfg(test)]
#[macro_use] extern crate variant_count;
