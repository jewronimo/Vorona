//! Focused diagnostic: unchanged fixed-phase C4 budget, followed by all C5 gates.
use vstone_core::{
    model::CoreInput, rectified, rectified_assembly, rectified_planes, surface::Surface,
};
fn main() {
    let a: Vec<_> = std::env::args().collect();
    let input: CoreInput = serde_json::from_str(&std::fs::read_to_string(&a[1]).unwrap()).unwrap();
    let surface = Surface::from_profile(&input.surface.profile)
        .unwrap()
        .for_pattern(input.params.pattern);
    let phase = a.get(2).map(|v| v.parse().unwrap()).unwrap_or(0.);
    let t = rectified::build_with_phase(&surface, &input.params, phase).unwrap();
    let g = rectified_planes::solve_reporting(&surface, &t, &input.params, |attempt| {
        eprintln!("{}", serde_json::to_string(attempt).unwrap())
    });
    match g {
        Ok(g) => {
            let x = rectified_assembly::assemble(&t, &g, &surface, &input.params).unwrap();
            println!(
                "{}",
                serde_json::json!({"phase":phase,"rows":x.outer_envelope_rows,"violations":x.violations,"solid_errors":x.solid_errors,"closure":x.closure,"pieces":x.pieces.len()})
            );
        }
        Err(e) => println!("{}", serde_json::to_string(&e).unwrap()),
    }
}
