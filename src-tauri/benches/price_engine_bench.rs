use app_lib::core::price_engine::{PriceEngine, PriceUpdate};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::time::Duration;

fn bench_single_update(c: &mut Criterion) {
    let engine = PriceEngine::new();

    c.bench_function("single_price_update", |b| {
        b.iter(|| {
            let update = PriceUpdate::new(
                black_box("SOL".to_string()),
                black_box(100.0),
                black_box(50000.0),
                black_box(5.5),
            );
            engine.process_update(update);
        });
    });
}

fn bench_batch_updates(c: &mut Criterion) {
    let engine = PriceEngine::new();

    for size in [100, 1000, 10000].iter() {
        c.bench_with_input(
            BenchmarkId::new("batch_price_updates", size),
            size,
            |b, &size| {
                b.iter(|| {
                    for i in 0..size {
                        let update = PriceUpdate::new(
                            black_box(format!("TOKEN{}", i % 10)),
                            black_box(100.0 + (i as f64 % 100.0)),
                            black_box(50000.0),
                            black_box(5.5),
                        );
                        engine.process_update(update);
                    }
                });
            },
        );
    }
}

fn bench_concurrent_updates(c: &mut Criterion) {
    use std::sync::Arc;
    use std::thread;

    let engine = Arc::new(PriceEngine::new());

    c.bench_function("concurrent_updates_4_threads", |b| {
        b.iter(|| {
            let mut handles = vec![];

            for thread_id in 0..4 {
                let engine_clone = Arc::clone(&engine);
                let handle = thread::spawn(move || {
                    for i in 0..250 {
                        let update = PriceUpdate::new(
                            black_box(format!("TOKEN{}_{}", thread_id, i % 10)),
                            black_box(100.0 + (i as f64 % 100.0)),
                            black_box(50000.0),
                            black_box(5.5),
                        );
                        engine_clone.process_update(update);
                    }
                });
                handles.push(handle);
            }

            for handle in handles {
                handle.join().unwrap();
            }
        });
    });
}

fn bench_get_metrics(c: &mut Criterion) {
    let engine = PriceEngine::new();

    // Populate with some data
    for i in 0..1000 {
        let update = PriceUpdate::new(
            format!("TOKEN{}", i % 10),
            100.0 + (i as f64 % 100.0),
            50000.0,
            5.5,
        );
        engine.process_update(update);
    }

    c.bench_function("get_metrics", |b| {
        b.iter(|| {
            black_box(engine.get_metrics());
        });
    });
}

fn bench_latency_target(c: &mut Criterion) {
    let engine = PriceEngine::new();

    let mut group = c.benchmark_group("latency_target");
    group.sample_size(1000);
    group.measurement_time(Duration::from_secs(10));

    // Target: <1ms (1000 microseconds) p95 latency
    group.bench_function("end_to_end_latency", |b| {
        b.iter(|| {
            let update = PriceUpdate::new(
                black_box("SOL".to_string()),
                black_box(100.0),
                black_box(50000.0),
                black_box(5.5),
            );
            engine.process_update(update);
        });
    });

    group.finish();

    // Verify metrics after benchmark
    let metrics = engine.get_metrics();
    println!("\n=== Latency Metrics ===");
    println!("P50: {:.2} μs", metrics.latency.p50);
    println!("P95: {:.2} μs", metrics.latency.p95);
    println!("P99: {:.2} μs", metrics.latency.p99);
    println!("Mean: {:.2} μs", metrics.latency.mean);
    println!("Messages processed: {}", metrics.messages_processed);
    println!("Throughput: {:.2} msg/s", metrics.throughput);

    // Assert p95 < 1ms (1000 μs)
    assert!(
        metrics.latency.p95 < 1000.0,
        "P95 latency ({:.2} μs) exceeds 1ms target",
        metrics.latency.p95
    );
}

criterion_group!(
    benches,
    bench_single_update,
    bench_batch_updates,
    bench_concurrent_updates,
    bench_get_metrics,
    bench_latency_target,
);
criterion_main!(benches);
