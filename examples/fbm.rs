//! An example of using the fBm noise function

extern crate noise;

use noise::{utils::*, Fbm, Seedable, MultiFractal};

fn main() {
    let mut fbm = Fbm::new().set_seed(12421343);

    PlaneMapBuilder::new(&fbm)
        .set_size(1000, 1000)
        .set_x_bounds(-10.0, 10.0)
        .set_y_bounds(-10.0, 10.0)
        .build().write_to_file("fbm/fbm.png");

    fbm = fbm.set_octaves(1);

    PlaneMapBuilder::new(&fbm)
        .set_size(1000, 1000)
        .set_x_bounds(-10.0, 10.0)
        .set_y_bounds(-10.0, 10.0)
        .build().write_to_file("fbm/fbm_octaves=1.png");

    fbm = fbm.set_octaves(2);

    PlaneMapBuilder::new(&fbm)
        .set_size(1000, 1000)
        .set_x_bounds(-10.0, 10.0)
        .set_y_bounds(-10.0, 10.0)
        .build().write_to_file("fbm/fbm_octaves=2.png");

    fbm = fbm.set_octaves(3);

    PlaneMapBuilder::new(&fbm)
        .set_size(1000, 1000)
        .set_x_bounds(-10.0, 10.0)
        .set_y_bounds(-10.0, 10.0)
        .build().write_to_file("fbm/fbm_octaves=3.png");

    fbm = fbm.set_octaves(4);

    PlaneMapBuilder::new(&fbm)
        .set_size(1000, 1000)
        .set_x_bounds(-10.0, 10.0)
        .set_y_bounds(-10.0, 10.0)
        .build().write_to_file("fbm/fbm_octaves=4.png");

    fbm = fbm.set_octaves(5);

    PlaneMapBuilder::new(&fbm)
        .set_size(1000, 1000)
        .set_x_bounds(-10.0, 10.0)
        .set_y_bounds(-10.0, 10.0)
        .build().write_to_file("fbm/fbm_octaves=5.png");
}
