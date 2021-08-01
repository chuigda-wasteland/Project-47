pub struct Defer<F: Fn() + Send + 'static> {
    f: F
}

impl<F: Fn() + Send + 'static> Defer<F> {
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<F: Fn() + Send + 'static> Drop for Defer<F> {
    fn drop(&mut self) {
        (self.f)()
    }
}
