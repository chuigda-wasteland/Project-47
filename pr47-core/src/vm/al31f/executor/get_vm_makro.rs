#[cfg(feature = "async")]
macro_rules! get_vm {
    ($thread:expr) => { $thread.vm.get_shared_data_mut() }
}

#[cfg(not(feature = "async"))]
macro_rules! get_vm {
    ($thread:expr) => { &mut $thread.vm }
}
