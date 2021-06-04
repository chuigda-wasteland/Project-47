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
