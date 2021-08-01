#[macro_export] macro_rules! boxed_slice {
    () => {
        vec![].into_boxed_slice()
    };
    ($($x:expr),+ $(,)?) => {
        vec![$($x),+].into_boxed_slice()
    };
}

#[macro_export] macro_rules! defer {
    ($func:expr) => {
        #[allow(unused_variables)]
        let deferred: crate::util::defer::Defer<_> = crate::util::defer::Defer::new($func);
    };
}
