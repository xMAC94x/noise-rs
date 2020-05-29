extern crate noise;

use noise::{utils::*, Checkerboard, Turbulence};

fn main() {
    let checkerboard = Checkerboard::new();
    let turbulence = Turbulence::new(&checkerboard);

    PlaneMapBuilder::new(&turbulence)
        .build()
        .write_to_file("turbulence.png");
}
