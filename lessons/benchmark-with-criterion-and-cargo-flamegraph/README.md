# Benchmark with Criterion and Cargo Flamegraph

Create a benchmark for the [Async Mutex](lessons/create-your-own-mutex/) we created and compare against Tokio's Mutex, Rust's std Mutex, and Parking Lot's Mutex and analyze the results.  Generate a flamegraph of the benchmarks.

First, let's install the first dependencies.  This will focus on using MAC OS, though instructions exist for [other operations systems](http://www.gnuplot.info/).

```shell
brew install gnuplot
cargo install criterion
cargo install flamegraph
```

No let's add the `benches` directory and the `bench.rs` file:

```shell
mkdir benches
touch benches/bench.rs
```

In the `Cargo.toml` file, add the following:

```toml
[dev-dependencies]
criterion = { version = "0.4", features = ["async_tokio", "html_reports"] }
parking_lot = "0.12.1"

[[bench]]
name = "bench"
harness = false

[profile.bench]
debug = true
```

We need `criterion` to run the benchmarks, and `parking_lot` as a comparision Mutex.  

In the `[[bench]]` section, we call out our `bench.rs` file as `name = "bench"`.  Setting `harness = false` disables `libtest` harness since Criterion provides it's own main function (e.g. `criterion_main!`).

For Flamegraph, we need to set `debug = true` in the  `[profile.bench]` section to allow the framegraph to be built off of the existing benchmarks.

Now that configuration is done, let's setup the first benchmark:

```rust
use criterion::{criterion_group, criterion_main, Criterion};

use std::sync::Arc;
use create_your_own_mutex::Mutex;

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

fn bench_our_mutex(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("our_mutex", move |b| {
        b.to_async(&rt)
            .iter(|| async { test_our_mutex() })
    });
}

criterion_group!(benches, bench_our_mutex);
criterion_main!(benches);
```

Similar to tests we wrote for the Mutex, the `test_our_mutex()` async function writes to the Mutex 100 times in a thread and 100 times outside of a thread.  This allows for simultaneous requests for writes to the Mutex.

The signature for `bench_our_mutex()` is fairly standard: `fn bench_our_mutex(c: &mut Criterion)`.  Since we need Tokio's runtime for the async function call, we have to manually invoke it.  The `c.bench_function()` does the heavy lifting.  Criterion uses the created iterator for benchmarking testing.

We need to mention the directory name as the first parameter in the `criterion_group!` macro.  The second parameter is the benchmark we just created.  All that's left is to add our `benches` group to the `criterion_main!` macro and run the tests:

```shell
cargo criterion
```

This will take about 10 - 30 seconds to complete.  You should then have the following terminal output:

```term
our_mutex               time:   [43.526 ns 43.663 ns 43.838 ns]
```

If we run the test again, it will compare against the first test:

```term
our_mutex               time:   [43.861 ns 44.415 ns 45.460 ns]
                        change: [-1.1772% -0.2303% +1.3138%] (p = 0.76 > 0.05)
                        No change in performance detected.
```

A chart was generated.  To access it:

```shell
open target/criterion/reports/index.html
```

Click on the `our_mutex` link to view the charts.

_TODO: explain charts_

That test was nice and will be a good guide for performance impacts on refactors, but we want to know how our Mutx stands up against other Mutexes.  Let's first pull in the other structs:

```rust
use parking_lot::Mutex as ParkingLotMutex;
use std::sync::Mutex as StdMutex;
use tokio::sync::Mutex as TokioMutex;
```

Now let's create a test for each one:

```rust
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
```

Criterion offers a nice feature to compare different tests in a single benchmark using groups.  Remove the `bench_our_mutex` function and replace with:

```rust
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
```

We first create a named group ("Mutex") as well as a separate runtime for each test.  We apply the same `bench_function` to the group for each test.  The last part is to replace `bench_our_mutex` in the `criterion_group!` macro with `compare_all` and run the tests again:

```term
Mutex/our_mutex         time:   [43.447 ns 43.521 ns 43.607 ns]
Mutex/tokio_mutex       time:   [46.370 ns 46.430 ns 46.496 ns]
Mutex/std_mutex         time:   [5.6855 ns 5.6943 ns 5.7047 ns]
Mutex/parking_lot_mutex time:   [5.6913 ns 5.7047 ns 5.7240 ns]
```

Nice, we now have a side-by-side comparision of all of the Mutexes.  Similar to last time, if we re-run the tests we get a comparison against the last run:

```term
Mutex/our_mutex         time:   [43.250 ns 43.287 ns 43.325 ns]
                        change: [-1.0062% -0.7389% -0.4853%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Mutex/tokio_mutex       time:   [46.339 ns 46.403 ns 46.492 ns]
                        change: [-0.1059% +0.1658% +0.4777%] (p = 0.26 > 0.05)
                        No change in performance detected.
Mutex/std_mutex         time:   [5.6655 ns 5.6828 ns 5.7041 ns]
                        change: [-0.9891% -0.6508% -0.3050%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Mutex/parking_lot_mutex time:   [5.6657 ns 5.6786 ns 5.6932 ns]
                        change: [-0.6826% -0.4153% -0.1593%] (p = 0.00 < 0.05)
                        Change within noise threshold.
```

New charts were generated.  Open the report again:

```shell
open target/criterion/reports/index.html
```

Click on the `Mutex` group link to view the side-by-side comparision charts.

_TODO: explain charts_