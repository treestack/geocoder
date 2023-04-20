extern crate criterion;
extern crate geocoder;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use geocoder::ReverseGeocoder;

pub fn criterion_benchmark(c: &mut Criterion) {
    let gc = ReverseGeocoder::from_file("../cities500.txt");
    let lat: f32 = 47.1;
    let lng: f32 = 11.0;
    c.bench_function("geocode 47.1/11.0", |b| {
        b.iter(|| gc.search(black_box(&lat), black_box(&lng), black_box(&1)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
