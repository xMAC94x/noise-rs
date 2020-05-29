extern crate criterion;
extern crate noise;
extern crate vek;

use criterion::*;
use noise::noisefield::{NoiseField2D, NoiseField3D};
use noise::{NoiseFn, Perlin};
use vek::{Vec2, Vec3};

criterion_group!(perlin_2d, bench_perlin2d);
criterion_group!(perlin_3d, bench_perlin3d);
criterion_main!(perlin_2d, perlin_3d);

fn bench_perlin2d(c: &mut Criterion) {
    let (width, height) = (128, 128);

    let perlin = Perlin::new();
    let mut field = NoiseField2D::new(width, height);
    field.build_field((-4.0, 4.0), (-4.0, 4.0));

    let mut group = c.benchmark_group("Perlin 2D");

    group.throughput(Throughput::Elements((width * height) as u64));

    group.bench_function("Single Points", |b| {
        b.iter(|| {
            for y in 0..height {
                for x in 0..width {
                    let coord = field.coord_at_point(Vec2 { x, y });
                    black_box(perlin.perlin_2d(coord.x, coord.y));
                }
            }
        })
    });

    group.bench_function("Single Points - Surflet", |b| {
        b.iter(|| {
            for y in 0..height {
                for x in 0..width {
                    let coord = field.coord_at_point(Vec2 { x, y });
                    black_box(perlin.perlin_2d_surflet([coord.x, coord.y]));
                }
            }
        })
    });

    group.bench_function("NoiseField - AutoVec", |b| {
        b.iter(|| black_box(perlin.perlin_2d_autovec(&field.x, &field.y)));
    });

    group.bench_function("NoiseField - AutoVec w/ ParIter", |b| {
        b.iter(|| black_box(perlin.perlin_2d_autovec_pariter(&field.x, &field.y)));
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
                        let coord = field.coord_at_point(Vec3 { x, y, z});
                        black_box(perlin.perlin_3d(coord.x, coord.y, coord.z));
                    }
                }
            }
        })
    });

    group.bench_function("NoiseField - AutoVec", |b| {
        b.iter(|| black_box(perlin.perlin_3d_autovec(&field.x, &field.y, &field.z)));
    });

    group.bench_function("NoiseField - AutoVec w/ ParIter", |b| {
        b.iter(|| black_box(perlin.perlin_3d_autovec_pariter(&field.x, &field.y, &field.z)));
    });

    group.finish();
}

// fn bench_perlin4(c: &mut Criterion) {
//     let perlin = Perlin::new();
//     c.bench_function("perlin 4d", |b| {
//         b.iter(|| perlin.get(black_box([42.0_f64, 37.0, 26.0, 128.0])))
//     });
// }
//
// fn bench_perlin4_64x64(c: &mut Criterion) {
//     let perlin = Perlin::new();
//     c.bench_function("perlin 4d (64x64)", |b| {
//         b.iter(|| {
//             for y in 0i8..64 {
//                 for x in 0i8..64 {
//                     black_box(perlin.get([x as f64, y as f64, x as f64, y as f64]));
//                 }
//             }
//         })
//     });
// }
