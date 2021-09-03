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

#[cfg(all(test, feature = "async"))]
use std::time::Duration;

#[cfg(any(test, feature = "bench"))]
use std::future::Future;

#[cfg(all(any(test, feature = "bench"), feature = "async-tokio"))]
pub fn block_on_future<F, R>(fut: F) -> R
    where F: Future<Output=R> + 'static
{
    tokio::runtime::Builder::new_current_thread().enable_time().build().unwrap().block_on(fut)
}

#[cfg(all(any(test, feature = "bench"), feature = "async-astd"))]
pub fn block_on_future<F, R>(fut: F) -> R
    where F: Future<Output=R> + 'static
{
    async_std::task::block_on(fut)
}

#[cfg(all(any(test, feature = "bench"), not(feature = "async")))]
pub fn block_on_future<F, R>(fut: F) -> R
    where F: Future<Output=R> + 'static
{
    pollster::block_on(fut)
}

#[cfg(all(test, feature = "async-tokio"))]
pub async fn testing_sleep(duration: Duration) {
    tokio::time::sleep(duration).await
}

#[cfg(all(test, feature = "async-astd"))]
pub async fn testing_sleep(duration: Duration) {
    async_std::task::sleep(duration).await
}
