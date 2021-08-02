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
        let deferred: Defer<_> = Defer::new($func);
    };
}
