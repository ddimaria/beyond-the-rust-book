# Create your own Async Mutex

We're going to learn about an Aynce Mutex by creating our own.  This Mutex is heavily inspired by [Tokio's Mutex](https://docs.rs/tokio/latest/tokio/sync/struct.Mutex.html).

Let's start by defined the `Mutex` struct:

```rust
pub struct Mutex<T> {
    inner: UnsafeCell<T>,
    semaphore: Semaphore,
}
```

This Mutex has a non-blocking lock that can be held across await points.  Async Mutex's are great for sharing IO across threads.  Fairness is guaranteed via a First In First Out (FIFO) approach.  A single-permit Semaphore is used to guarantee that only one lock is acquired at any given time.  If a panic occurs on a thread, this Mutex isn't poisened like the std::sync::Mutex, the lock is just released. 

This Mutex is intended to be wrapped in an Arc when sending across threads:

```rust
let mutex = Arc::new(Mutex::new(0));
```

A `MutexGuard` represents exclusive access to the inner value (`T`).  Since this struct is Send, it can be held across await points.  We're currently not using the `_permit` field.

```rust
pub struct MutexGuard<'a, T> {
    lock: &'a Mutex<T>,
    _permit: SemaphorePermit<'a>,
}
```

`Send` and `Sync` are marker traits, which simply means they are traits with empty bodies.  The `Mutex` and `MutexGuard` are both safe to send between threads and share between threads.  The Sync trait needs to be implemented since UnsafeCell isn't Sync.  Sync is necessary to protect direct access to T.   

```rust
unsafe impl<T> Sync for Mutex<T> where T: Send + Sync {}
unsafe impl<T> Sync for MutexGuard<'_, T> where T: Send + Sync {}
```

Now that the structs are defined, we can implement `new()` and `lock()` on the Mutex:

```rust
impl<T> Mutex<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner: UnsafeCell::new(inner),
            semaphore: Semaphore::new(1),
        }
    }

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
```

The `inner` part of the Mutex is the actual data we're allowing shared mutable access to.  This is possible because of Rust's [UnsafeCell](https://doc.rust-lang.org/stable/std/cell/struct.UnsafeCell.html) function.  This function opts out of the immutability guarantee of shared references and is the core primitive in interior mutability.

The `semaphore` is used to protect the `inner` data from shared access to it.  While a Semphore can be used to handle multiple permits, we're only issuing a single permit to represent exclusive access.

The `lock` function allows us to acquire a lock and return an exclusive MutexGuard.  The Semaphore is the superpower in this Mutex as it asynchronously waits if a permit isn't available.  Permits are issued on a first come, first serve basis.  We should always be able to acquire a permit since we never close the Semaphore.

You may have noticed that we're holding onto `_permit`.  This is because we'll drop that permit early if not.  However, when a `MutexGuard` is dropped, the permit is released.

With core functions defined, we can now implement `Deref` and `DerefMut` for exposing read and write access to the inner data.

```rust
impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.inner.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.inner.get() }
    }
}
```

That's all we need to implement to have a functioning Async Mutex.  It's a simple data structure and we lean heavily on Tokio's Semaphore to guarantee exclusive access.

Let's put this all together in a single test:

```rust
async fn async_mutex_across_threads() {
    let mut set = JoinSet::new();
    let count = Arc::new(Mutex::new(0));

    // create 10 threads and increment the Mutex's inner value 100 times per thread
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
```