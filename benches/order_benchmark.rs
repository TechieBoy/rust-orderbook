use core::time::Duration;
use criterion::{criterion_group, criterion_main, Criterion, SamplingMode};
use orderbooklib::{OrderBook, Side};
use rand::Rng;
use rand_distr::{Distribution, Normal};

fn initialize_orderbook(
    num_orders: i32,
    rng: &mut rand::prelude::ThreadRng,
    normal: Normal<f64>,
) -> OrderBook {
    let mut ob = OrderBook::new("Random".to_string());
    for _ in 0..num_orders {
        if rng.gen_bool(0.5) {
            ob.add_limit_order(Side::Bid, normal.sample(rng) as u64, rng.gen_range(1..=500));
        } else {
            ob.add_limit_order(Side::Ask, normal.sample(rng) as u64, rng.gen_range(1..=500));
        }
    }
    ob
}

fn match_orders(ob: &mut OrderBook, rng: &mut rand::prelude::ThreadRng, normal: Normal<f64>) {
    for _ in 0..10000 {
        let _fr = ob.add_limit_order(Side::Ask, normal.sample(rng) as u64, rng.gen_range(1..=500));
    }
    //ob.get_bbo();
}
pub fn criterion_benchmark(c: &mut Criterion) {
    let mut rng: rand::prelude::ThreadRng = rand::thread_rng();
    let normal = Normal::new(5000.0, 500.0).unwrap();
    let mut ob = initialize_orderbook(100_000, &mut rng, normal);
    let mut group = c.benchmark_group("order-benchmark");
    group.sample_size(10);
    group.measurement_time(Duration::new(20, 0));
    group.bench_function("Match 100k orders", |b| {
        b.iter(|| initialize_orderbook(1_000_00, &mut rng, normal))
    });
    /*
    group.bench_function("match 10000 orders on orderbook with 100k orders", |b| {
        b.iter(|| match_orders(&mut ob, &mut rng, normal))
    });
    */
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
