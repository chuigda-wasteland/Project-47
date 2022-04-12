#[cfg(feature = "al31fu")]
fn main() {
    cc::Build::new()
        .include("src/vm/al31fu/cxx/include")
        .file("src/vm/al31fu/cxx/src/insc.cc")
        .use_plt(false)
        .try_compile("al31fu")
        .unwrap();
}

#[cfg(not(feature = "al31fu"))] fn main() {}
