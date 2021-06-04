use std::error::Error;

pub enum UncheckedException {
    DummyException(i32)
}

pub enum Exception {
    UncheckedException(UncheckedException),
    CheckedException(Box<dyn 'static + Error + Send>)
}
