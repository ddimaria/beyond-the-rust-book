use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use tokio::sync::{Semaphore, SemaphorePermit};

/// Async Mutex modeled from Tokio's Mutex.  This is a non-blocking lock that
/// can be held across await points.  Async Mutex's are great for sharing IO
/// across threads.  This Mutex is intended to be wrapped in an Arc when sending
/// across threads.  Fairness is guaranteed via a First In First Out (FIFO)
/// approach.  A single-permit Semaphore is used to guarantee that only one
/// lock is acquired at any given time.  If a panic occurs on a thread, this
/// Mutex isn't poisened like the std::sync::Mutex, the lock is just released.
pub struct Mutex<T> {
    inner: UnsafeCell<T>,
    semaphore: Semaphore,
}

/// A MutexGuard represents exclusive access to the inner value (`T`).  Since
/// this struct is Send, it can be held across await points.
pub struct MutexGuard<'a, T> {
    lock: &'a Mutex<T>,
    _permit: SemaphorePermit<'a>,
}

/// Send and Sync are marker traits, which simply means they are traits with    
/// empty bodies.  The Mutex and MutexGuard are both safe to send between
/// threads and share between threads.  The Sync trait needs to be implemented
/// since UnsafeCell isn't Sync.  Sync is necessary to protect direct access
/// to T.   
unsafe impl<T> Sync for Mutex<T> where T: Send + Sync {}
unsafe impl<T> Sync for MutexGuard<'_, T> where T: Send + Sync {}

impl<T> Mutex<T> {
    /// Initial state is unlicked.  Issue a single-permit Semaphore to provide
    /// exclusive access.
    pub fn new(inner: T) -> Self {
        Self {
            inner: UnsafeCell::new(inner),
            semaphore: Semaphore::new(1),
        }
    }

    /// Acquire a lock and return an exclusive MutexGuard.  The Semaphore is
    /// the superpower in this Mutex as it asynchronously waits if a permit
    /// isn't available.  Permits are issued on a first come, first serve
    /// basis.  We're ignoring the error state in `acquire()` since we never
    /// close the Semaphore.
    pub async fn lock(&self) -> MutexGuard<'_, T> {
        let _permit = self
            .semaphore
            .acquire()
            .await
            .unwrap_or_else(|_| unreachable!());

        MutexGuard {
            lock: self,
            _permit,
        }
    }
}

/// Expose the inner value of the Mutex for reading.
impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.inner.get() }
    }
}

/// Expose the inner value of the Mutex for reading and writing.
impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.inner.get() }
    }
}

#[cfg(test)]
mod tests {
    use tokio::task::JoinSet;

    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn it_works_across_a_single_thread() {
        let data1 = Arc::new(Mutex::new(0));
        let data2 = Arc::clone(&data1);

        assert_eq!(*data1.clone().lock().await, 0);

        let handle = tokio::spawn(async move {
            *data2.lock().await += 1;
        });

        *data1.lock().await += 1;
        handle.await.unwrap();

        assert_eq!(*data1.clone().lock().await, 2);
    }

    #[tokio::test]
    async fn it_does_not_lock_with_fast_writes_across_multiple_threads() {
        let mut set = JoinSet::new();
        let count = Arc::new(Mutex::new(0));

        for _ in 0..10 {
            let my_count = Arc::clone(&count);

            set.spawn(async move {
                for _ in 0..100 {
                    *my_count.lock().await += 1;
                }
            });
        }

        // await all threads to complete
        while let Some(_) = set.join_next().await {}

        assert_eq!(*count.clone().lock().await, 1000);
    }
}
