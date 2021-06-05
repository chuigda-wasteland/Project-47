//! # `async_utils.rs`: Re-exports asynchronous structures from `tokio`, `async-std` and `futures`

pub use futures::future::join_all;

#[cfg(feature = "async-tokio")]
pub use tokio::{
    sync::{
        Mutex,
        MutexGuard,
        oneshot
    },
    task,
    task::yield_now
};

#[cfg(feature = "async-astd")]
pub use async_std::{
    sync::{ Mutex, MutexGuard },
    task,
    task::yield_now
};

#[cfg(feature = "async-astd")]
pub use futures::channel::oneshot;
