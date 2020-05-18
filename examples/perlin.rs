//! An example of using perlin noise

extern crate noise;

use noise::{utils::*, Perlin, Seedable};

fn main() {
    let perlin = Perlin::new();

    PlaneMapBuilder::new(&perlin)
        .set_size(1000, 1000)
        .set_x_bounds(-10.0, 10.0)
        .set_y_bounds(-10.0, 10.0)
        .build()
        .write_to_file("perlin.png");

    let perlin = perlin.set_seed(1);

    PlaneMapBuilder::new(&perlin)
        .set_size(1000, 1000)
        .set_x_bounds(-10.0, 10.0)
        .set_y_bounds(-10.0, 10.0)
        .build()
        .write_to_file("perlin_seed=1.png");

    let perlin = perlin.set_seed(2);

    PlaneMapBuilder::new(&perlin)
        .set_size(1000, 1000)
        .set_x_bounds(-10.0, 10.0)
        .set_y_bounds(-10.0, 10.0)
        .build()
        .write_to_file("perlin_seed=2.png");
}
