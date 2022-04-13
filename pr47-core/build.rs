#[cfg(feature = "al31fu")]
fn main() {
    use cc::Build;

    let mut build: Build = Build::new();

    build.cpp(true);
    build.include("src/vm/al31fu/cxx/include");
    build.file("src/vm/al31f/cxx/src/insc/cc");
    build.use_plt(false);
    build.flag_if_supported("-fno-rtti");
    build.flag_if_supported("-fno-exceptions");
    build.flag_if_supported("-fno-unwind-tables");
    build.flag_if_supported("-fno-asynchronous-unwind-tables");
    build.flag_if_supported("/utf-8");

    #[cfg(target_pointer_width = "64")]
    {
        build.flag_if_supported("-m64");
        build.define("PR47_AL31FU_64BIT", None);
    }
    #[cfg(target_pointer_width = "32")]
    {
        build.flag_if_supported("-m32");
        build.define("PR47_AL31FU_32BIT", None);
    }

    cc::Build::new()
        .cpp(true)
        .include("src/vm/al31fu/cxx/include")
        .file("src/vm/al31fu/cxx/src/insc.cc")
        .use_plt(false)
        .try_compile("al31fu")
        .unwrap();
}

#[cfg(not(feature = "al31fu"))] fn main() {}
