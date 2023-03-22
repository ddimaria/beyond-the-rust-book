use std::sync::Arc;

use criterion::{criterion_group, criterion_main, Criterion};

use create_your_own_mutex::Mutex;
use parking_lot::Mutex as ParkingLotMutex;
use std::sync::Mutex as StdMutex;
use tokio::sync::Mutex as TokioMutex;

async fn test_our_mutex() {
    let mutex = Arc::new(Mutex::new(0));
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
    assert_eq!(*mutex.lock().await, 200);
}

async fn test_tokio_mutex() {
    let mutex = Arc::new(TokioMutex::new(0));
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
    assert_eq!(*mutex.lock().await, 200);
}

async fn test_std_mutex() {
    let mutex = Arc::new(StdMutex::new(0));
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
}

async fn test_parking_lot_mutex() {
    let mutex = Arc::new(ParkingLotMutex::new(0));
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
}


fn compare_all(c: &mut Criterion) {
    let mut group = c.benchmark_group("Mutex");
    let rt_ours = tokio::runtime::Runtime::new().unwrap();
    let rt_tokio = tokio::runtime::Runtime::new().unwrap();
    let rt_std = tokio::runtime::Runtime::new().unwrap();
    let rt_parking_lot = tokio::runtime::Runtime::new().unwrap();

    group.bench_function("our_mutex", move |b| {
        b.to_async(&rt_ours)
            .iter(|| async { test_our_mutex() })
    });

    group.bench_function("tokio_mutex", move |b| {
        b.to_async(&rt_tokio)
            .iter(|| async { test_tokio_mutex() })
    });

    group.bench_function("std_mutex", move |b| {
        b.to_async(&rt_std)
            .iter(|| async { test_std_mutex() })
    });

    group.bench_function("parking_lot_mutex", move |b| {
        b.to_async(&rt_parking_lot)
            .iter(|| async { test_parking_lot_mutex() })
    });
}

criterion_group!(benches, compare_all);
criterion_main!(benches);
