extern crate noise;

use noise::{utils::*, Checkerboard, Perlin, Turbulence};

fn main() {
    let perlin = Perlin::new();
    let cboard = Checkerboard::new();
    let turbulence = Turbulence::new(&cboard);

    PlaneMapBuilder::new(&turbulence)
        .build()
        .write_to_file("turbulence.png");
}
