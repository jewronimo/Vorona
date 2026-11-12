//! Export native production inputs for the identical-input SciPy comparison.
use vstone_core::{
    model::CoreInput, planes, seeds, sidecount, surface::Surface, topology::Topology,
};
fn main() {
    let mut input: CoreInput = serde_json::from_str(include_str!(
        "../../tests/fixtures/core_input_cylinder.json"
    ))
    .unwrap();
    input.params.target_size = 6.5;
    input.params.size_variation = 0.15;
    input.params.depth = 1.5;
    input.params.randomness = 0.;
    input.params.seed = 1;
    let surface = Surface::from_profile(&input.surface.profile).unwrap();
    // Reproduce the preceding cylinder construction for a controlled old/new
    // lattice comparison; all fit code remains shared with production.
    let legacy = std::env::args().any(|a| a == "legacy");
    let t = if legacy {
        let pitch = std::f64::consts::TAU * 9. / 8.;
        let h = pitch * 3f64.sqrt() / 2.;
        let intervals = (surface.length() / h).floor() as usize;
        let stretched = surface.length() / intervals as f64;
        let mut sites = Vec::new();
        for row in 0..intervals + 5 {
            let s = (row as f64 - 2.) * stretched;
            for col in 0..8 {
                sites.push(seeds::Seed {
                    u: (col as f64 + 0.5 * (row % 2) as f64) * std::f64::consts::TAU / 8.,
                    v: seeds::extended_v(&surface, s),
                    s,
                    row,
                    auxiliary: s < 0. || s > surface.length(),
                    depth: seeds::symmetric_depth(row, input.params.depth),
                });
            }
        }
        Topology::build(sites).unwrap()
    } else {
        sidecount::build(&surface, &input.params).unwrap()
    };
    println!(
        "{}",
        planes::export_native(&surface, &t, &input.params).unwrap()
    );
}
