//! An example of using simplex noise

extern crate noise;

use noise::{utils::*, OpenSimplex, ScaleBias, Seedable};

fn main() {
    let open_simplex = OpenSimplex::new();

    PlaneMapBuilder::new(&open_simplex)
        .build()
        .write_to_file("open_simplex.png");

    let open_simplex = open_simplex.set_seed(1);

    let scaled = ScaleBias::new(&open_simplex).set_scale(2.0);

    let noise_map = PlaneMapBuilder::new(&scaled)
        .set_size(1024, 1024)
        .set_x_bounds(-2.0, 2.0)
        .set_y_bounds(-2.0, 2.0)
        .build();

    ImageRenderer::new()
        .set_gradient(ColorGradient::new().build_rainbow_gradient())
        .render(&noise_map)
        .write_to_file("open_simplex_rainbow.png");
}
