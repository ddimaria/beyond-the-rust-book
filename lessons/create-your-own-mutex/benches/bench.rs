use std::sync::Arc;

use criterion::{criterion_group, criterion_main, Criterion};

use create_your_own_mutex::Mutex;
use parking_lot::Mutex as ParkingLotMutex;
use std::sync::Mutex as StdMutex;
use tokio::sync::Mutex as TokioMutex;

macro_rules! test_async_mutex {
    ($mutex:expr) => {{
        let mutex = $mutex;
        let cloned = Arc::clone(&mutex);

        let handle = tokio::spawn(async move {
            for _ in 0..100 {
                *cloned.lock().await += 1;
            }
        });

        for _ in 0..100 {
            *mutex.lock().await += 1;
        }

        handle.await.unwrap();

        // assert_eq!(*mutex.lock().await, 200);
    }};
}

macro_rules! test_std_mutex {
    ($mutex:expr) => {{
        let mutex = $mutex;
        let cloned = Arc::clone(&mutex);

        let handle = tokio::spawn(async move {
            for _ in 0..100 {
                *cloned.lock().unwrap() += 1;
            }
        });

        for _ in 0..100 {
            *mutex.lock().unwrap() += 1;
        }

        handle.await.unwrap();

        assert_eq!(*mutex.lock().unwrap(), 200);
    }};
}

macro_rules! test_parking_lot_mutex {
    ($mutex:expr) => {{
        let mutex = $mutex;
        let cloned = Arc::clone(&mutex);

        let handle = tokio::spawn(async move {
            for _ in 0..100 {
                *cloned.lock() += 1;
            }
        });

        for _ in 0..100 {
            *mutex.lock() += 1;
        }

        handle.await.unwrap();

        assert_eq!(*mutex.lock(), 200);
    }};
}

fn bench_ours(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("our_mutex", move |b| {
        b.to_async(&rt)
            .iter(|| async { test_async_mutex!(Arc::new(Mutex::new(0))) })
    });
}

fn bench_tokio(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("tokio_mutex", move |b| {
        b.to_async(&rt)
            .iter(|| async { test_async_mutex!(Arc::new(TokioMutex::new(0))) })
    });
}

fn bench_std(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("std_mutex", move |b| {
        b.to_async(&rt)
            .iter(|| async { test_std_mutex!(Arc::new(StdMutex::new(0))) })
    });
}

fn bench_parking_lot(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("parking_lot_mutex", move |b| {
        b.to_async(&rt)
            .iter(|| async { test_parking_lot_mutex!(Arc::new(ParkingLotMutex::new(0))) })
    });
}

criterion_group!(
    benches,
    bench_ours,
    bench_tokio,
    bench_std,
    bench_parking_lot
);
criterion_main!(benches);
