#[macro_use]
extern crate criterion;
extern crate noise;

use criterion::*;
use noise::noisefield::{NoiseField2D, NoiseField3D};
use noise::{NoiseFieldFn, NoiseFn, Perlin};

criterion_group!(perlin_2d, bench_perlin2d);
criterion_group!(perlin_3d, bench_perlin3d);
criterion_main!(perlin_2d, perlin_3d);

fn bench_perlin2d(c: &mut Criterion) {
    let (width, height) = (32, 32);

    let perlin = Perlin::new();
    let mut field = NoiseField2D::new(width, height);
    field.build_field((-4.0, 4.0), (-4.0, 4.0));

    let mut group = c.benchmark_group("Perlin 2D");

    group.throughput(Throughput::Elements((width * height) as u64));

    group.bench_function("Single Points", |b| {
        b.iter(|| {
            for y in 0..height {
                for x in 0..width {
                    black_box(perlin.get(field.coord_at_point([x, y])));
                }
            }
        })
    });

    group.bench_function("NoiseField - Serial", |b| {
        b.iter(|| perlin.process_2d_field_serial(black_box(&field)))
    });

    group.bench_function("NoiseField - Parallel", |b| {
        b.iter(|| perlin.process_2d_field_parallel(black_box(&field)))
    });

    group.finish();
}

fn bench_perlin3d(c: &mut Criterion) {
    let (width, height, depth) = (32, 32, 32);

    let perlin = Perlin::new();
    let mut field = NoiseField3D::new(width, height, depth);
    field.build_field((-4.0, 4.0), (-4.0, 4.0), (-4.0, 4.0));

    let mut group = c.benchmark_group("Perlin 3D");

    group.throughput(Throughput::Elements((width * height * depth) as u64));

    group.bench_function("Single Points", |b| {
        b.iter(|| {
            for z in 0..depth {
                for y in 0..height {
                    for x in 0..width {
                        black_box(perlin.get(field.coord_at_point([x, y, z])));
                    }
                }
            }
        })
    });

    group.bench_function("NoiseField - Serial", |b| {
        b.iter(|| perlin.process_3d_field_serial(black_box(&field)))
    });

    group.bench_function("NoiseField - Parallel", |b| {
        b.iter(|| perlin.process_3d_field_parallel(black_box(&field)))
    });

    group.finish();
}

fn bench_perlin2_field_serial(c: &mut Criterion) {
    let perlin = Perlin::new();
    let mut field = NoiseField2D::new(100, 100);
    field.build_field((-4.0, 4.0), (-4.0, 4.0));
    c.bench_function("perlin 2d field - serial", |b| {
        b.iter(|| perlin.process_2d_field_serial(black_box(&field)))
    });
}

fn bench_perlin2_field_parallel(c: &mut Criterion) {
    let perlin = Perlin::new();
    let mut field = NoiseField2D::new(100, 100);
    field.build_field((-4.0, 4.0), (-4.0, 4.0));
    c.bench_function("perlin 2d field - parallel", |b| {
        b.iter(|| perlin.process_2d_field_parallel(black_box(&field)))
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
