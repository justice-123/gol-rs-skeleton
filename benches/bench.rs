use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use gol_rs::gol::{Params, event::Event, self};
use sdl2::keyboard::Keycode;
use tokio::sync::mpsc;

fn bench_gol(c: &mut Criterion) {
    let mut group = c.benchmark_group("Gol");
    group
        .sampling_mode(criterion::SamplingMode::Flat)
        .sample_size(10);
    for thread in 1..=num_cpus::get() {
        group.bench_with_input(BenchmarkId::new("Threads", thread), &thread, |bencher, thread| {
            let params = Params {
                turns: 1000,
                threads: *thread,
                image_width: 512,
                image_height: 512,
            };
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            bencher.to_async(runtime).iter(|| async {
                let (events_tx, mut events_rx) = mpsc::channel::<Event>(1000);
                let (_key_presses_tx, key_presses_rx) = mpsc::channel::<Keycode>(10);
                tokio::spawn(gol::run(params, events_tx, key_presses_rx));
                loop {
                    if events_rx.recv().await.is_none() {
                        break;
                    }
                }
            })
        });

    }
    group.finish();
}


criterion_group!(benches, bench_gol);
criterion_main!(benches);
