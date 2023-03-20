pub mod mutex;

// use std::cell::UnsafeCell;
// use std::fmt;
// use std::ops::{Deref, DerefMut};
// use std::sync::{LockResult, TryLockError, TryLockResult};

// pub struct Mutex<T: ?Sized> {
//     data: UnsafeCell<T>,
// }

// // unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}
// // unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}

// pub struct MutexGuard<'a, T: ?Sized + 'a> {
//     lock: &'a Mutex<T>,
// }

// // impl<T: ?Sized> !Send for MutexGuard<'_, T> {}
// // unsafe impl<T: ?Sized + Sync> Sync for MutexGuard<'_, T> {}

// impl<T> Mutex<T> {
//     pub const fn new(t: T) -> Mutex<T> {
//         Mutex {
//             data: UnsafeCell::new(t),
//         }
//     }
// }

// impl<T: ?Sized> Mutex<T> {
//     pub fn lock(&self) -> LockResult<MutexGuard<'_, T>> {
//         unsafe { MutexGuard::new(self) }
//     }
//     pub fn try_lock(&self) -> TryLockResult<MutexGuard<'_, T>> {
//         unsafe {
//             if self.inner.try_lock() {
//                 Ok(MutexGuard::new(self)?)
//             } else {
//                 Err(TryLockError::WouldBlock)
//             }
//         }
//     }

//     pub fn unlock(guard: MutexGuard<'_, T>) {
//         drop(guard);
//     }

//     pub fn into_inner(self) -> LockResult<T>
//     where
//         T: Sized,
//     {
//         let data = self.data.into_inner();
//         poison::map_result(self.poison.borrow(), |()| data)
//     }

//     pub fn get_mut(&mut self) -> LockResult<&mut T> {
//         let data = self.data.get_mut();
//         poison::map_result(self.poison.borrow(), |()| data)
//     }
// }

// impl<T> From<T> for Mutex<T> {
//     /// Creates a new mutex in an unlocked state ready for use.
//     /// This is equivalent to [`Mutex::new`].
//     fn from(t: T) -> Self {
//         Mutex::new(t)
//     }
// }

// impl<T: ?Sized + Default> Default for Mutex<T> {
//     /// Creates a `Mutex<T>`, with the `Default` value for T.
//     fn default() -> Mutex<T> {
//         Mutex::new(Default::default())
//     }
// }

// impl<T: ?Sized + fmt::Debug> fmt::Debug for Mutex<T> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         let mut d = f.debug_struct("Mutex");
//         match self.try_lock() {
//             Ok(guard) => {
//                 d.field("data", &&*guard);
//             }
//             Err(TryLockError::Poisoned(err)) => {
//                 d.field("data", &&**err.get_ref());
//             }
//             Err(TryLockError::WouldBlock) => {
//                 struct LockedPlaceholder;
//                 impl fmt::Debug for LockedPlaceholder {
//                     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//                         f.write_str("<locked>")
//                     }
//                 }
//                 d.field("data", &LockedPlaceholder);
//             }
//         }
//         d.field("poisoned", &self.poison.get());
//         d.finish_non_exhaustive()
//     }
// }

// impl<'mutex, T: ?Sized> MutexGuard<'mutex, T> {
//     unsafe fn new(lock: &'mutex Mutex<T>) -> LockResult<MutexGuard<'mutex, T>> {
//         poison::map_result(lock.poison.guard(), |guard| MutexGuard {
//             lock,
//             poison: guard,
//         })
//     }
// }

// impl<T: ?Sized> Deref for MutexGuard<'_, T> {
//     type Target = T;

//     fn deref(&self) -> &T {
//         unsafe { &*self.lock.data.get() }
//     }
// }

// impl<T: ?Sized> DerefMut for MutexGuard<'_, T> {
//     fn deref_mut(&mut self) -> &mut T {
//         unsafe { &mut *self.lock.data.get() }
//     }
// }

// impl<T: ?Sized> Drop for MutexGuard<'_, T> {
//     fn drop(&mut self) {
//         unsafe {
//             self.lock.poison.done(&self.poison);
//             self.lock.inner.unlock();
//         }
//     }
// }

// // impl<T: ?Sized + fmt::Debug> fmt::Debug for MutexGuard<'_, T> {
// //     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
// //         fmt::Debug::fmt(&**self, f)
// //     }
// // }

// // impl<T: ?Sized + fmt::Display> fmt::Display for MutexGuard<'_, T> {
// //     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
// //         (**self).fmt(f)
// //     }
// // }

// pub fn guard_lock<'a, T: ?Sized>(guard: &MutexGuard<'a, T>) -> &'a sys::Mutex {
//     &guard.lock.inner
// }

// pub fn guard_poison<'a, T: ?Sized>(guard: &MutexGuard<'a, T>) -> &'a poison::Flag {
//     &guard.lock.poison
// }

fn main() {
    println!("Hello, world!");
}
