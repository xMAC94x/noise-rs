#[macro_use]
extern crate criterion;
extern crate noise;

use criterion::{black_box, Criterion};
use noise::{NoiseFn, Perlin, NoiseFieldFn};
use noise::noisefield::NoiseField2D;

criterion_group!(perlin, bench_perlin2, bench_perlin3, bench_perlin4);
criterion_group!(
    perlin_64x64,
    bench_perlin2_64x64,
    bench_perlin3_64x64,
    bench_perlin4_64x64
);
criterion_group!(perlin_field, bench_perlin2_field);
criterion_group!(perlin_1kx1k, bench_perlin2_1kx1k);
// criterion_main!(perlin, perlin_64x64, perlin_field);
criterion_main!(perlin_field, perlin_1kx1k);

fn bench_perlin2(c: &mut Criterion) {
    let perlin = Perlin::new();
    c.bench_function("perlin 2d", |b| {
        b.iter(|| perlin.get(black_box([42.0_f64, 37.0])))
    });
}

fn bench_perlin2_field(c: &mut Criterion) {
    let perlin = Perlin::new();
    let mut field = NoiseField2D::new(1000, 1000);
    field.build_field((-4.0, 4.0), (-4.0, 4.0));
    c.bench_function("perlin 2d field", |b| {
        b.iter(|| perlin.process_field(black_box(&field)))
    });
}

fn bench_perlin3(c: &mut Criterion) {
    let perlin = Perlin::new();
    c.bench_function("perlin 3d", |b| {
        b.iter(|| perlin.get(black_box([42.0_f64, 37.0, 26.0])))
    });
}

fn bench_perlin4(c: &mut Criterion) {
    let perlin = Perlin::new();
    c.bench_function("perlin 4d", |b| {
        b.iter(|| perlin.get(black_box([42.0_f64, 37.0, 26.0, 128.0])))
    });
}

fn bench_perlin2_64x64(c: &mut Criterion) {
    let perlin = Perlin::new();
    c.bench_function("perlin 2d (64x64)", |b| {
        b.iter(|| {
            for y in 0i8..64 {
                for x in 0i8..64 {
                    black_box(perlin.get([x as f64, y as f64]));
                }
            }
        })
    });
}

fn bench_perlin2_1kx1k(c: &mut Criterion) {
    let perlin = Perlin::new();
    let mut field = NoiseField2D::new(1000, 1000);
    field.build_field((-4.0, 4.0), (-4.0, 4.0));
    c.bench_function("perlin 2d (64x64)", |b| {
        b.iter(|| {
            for y in 0i16..1000 {
                for x in 0i16..1000 {
                    black_box(perlin.get(field.coord_at_point([x as usize, y as usize])));
                }
            }
        })
    });
}

fn bench_perlin3_64x64(c: &mut Criterion) {
    let perlin = Perlin::new();
    c.bench_function("perlin 3d (64x64)", |b| {
        b.iter(|| {
            for y in 0i8..64 {
                for x in 0i8..64 {
                    black_box(perlin.get([x as f64, y as f64, x as f64]));
                }
            }
        })
    });
}

fn bench_perlin4_64x64(c: &mut Criterion) {
    let perlin = Perlin::new();
    c.bench_function("perlin 4d (64x64)", |b| {
        b.iter(|| {
            for y in 0i8..64 {
                for x in 0i8..64 {
                    black_box(perlin.get([x as f64, y as f64, x as f64, y as f64]));
                }
            }
        })
    });
}
