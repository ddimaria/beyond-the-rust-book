# Beyond the Rust Book

The [Rust Book](https://doc.rust-lang.org/book/) is an incredible resource.  Combined with [Rustlings](https://github.com/rust-lang/rustlings), a developer can learn quite about about the Rust programming language.  I created this repo for those of you who want examples on Rust concepts that go a bit deeper in some areas and delve into third-party crates for some common use cases.

## Lessons

### Quick Links
* [Tokio Pub Sub](lessons/tokio-pub-sub/)
* [Create your own Async Mutex](lessons/create-your-own-mutex/)

### Tokio Pub Sub

Build a small pubsub system with subscribers running on separate threads.  The publisher can send text messages to all subscribers, as well as signal them to gracefully shutdown. 

[Tokio Pub Sub](lessons/tokio-pub-sub/)

### Create your own Async Mutex

Learn about an Async Mutex by creating your own.  This Mutex is heavily inspired by [Tokio's Mutex](https://docs.rs/tokio/latest/tokio/sync/struct.Mutex.html).

[Create your own Async Mutex](lessons/create-your-own-mutex/)

## Lessons to Write
* Benchmarking with Criterion
* Performance Analysis with cargo-flamegraph and DHAT
* Basic GitHub Actions CI Setup (format, clippy, test, codecov, cargo-udeps, cargo-deny)
* Comparing `iter()` to `into_iter()`
* Testing with cargo-nextest
* Optimizing Rust Code: Basic
* Optimizing Rust Code: Avoiding Allocations
* Optimizing Rust Code: Parallelizing with Rayon and Crossbeam 
* Optimizing Rust Code: Inlining Functions
* Implementing io_uring in Rust 
* Write a Derive Procedural Macro for the NewType Pattern
* Execute Code in Wasmtime Runtime
* Basic Cryptography