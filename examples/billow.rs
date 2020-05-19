//! An example of using the Billow noise function

extern crate noise;

use noise::{utils::*, Billow};

fn main() {
    PlaneMapBuilder::new(&Billow::new())
        .build()
        .write_to_file("billow.png");

    CylinderMapBuilder::new(&Billow::new())
        .build()
        .write_to_file("billow-cylinder.png");

    SphereMapBuilder::new(&Billow::new())
        .build()
        .write_to_file("billow-sphere.png");
}
