/// # Pr47(Project-47): A semi-experimental embeddable programming language for Rust
///
/// ## ⚠️⚠️⚠️ Develop stage note ⚠️⚠️⚠
/// By this time the author doesn't know which APIs are necessary for user to write low-level,
/// direct FFI bindings, nor does the author know which APIs are necessary for user to tweak the
/// VM runtime. So we are making as many things public as possible.

pub mod data;
pub mod ds;
pub mod ffi;
pub mod sema;
pub mod syntax;
pub mod util;
pub mod vm;

#[cfg(all(feature = "async-astd", feature = "async-tokio"))]
compile_error!("features `async-astd` and `async-tokio` are mutually exclusive");

#[cfg(test)]
mod test {
    #[test]
    fn test() {
    }
}
