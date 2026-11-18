//! A06 native C3-C5 physical-band diagnostics; C6/C7 manufacturing gates held.
use vstone_core::{
    model::{CoreInput, Pattern},
    rectified, rectified_assembly, rectified_planes,
    surface::Surface,
};
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let mut input: CoreInput =
        serde_json::from_str(&std::fs::read_to_string(&args[1]).unwrap()).unwrap();
    input.params.pattern = Pattern::VoronoiTriangles;
    input.params.min_triangle_edge = 1.5;
    input.params.max_polygon_edge = 3.;
    input.params.count_step = 2;
    input.params.relief_base = Default::default();
    input.params.min_edge = 1.;
    input.params.min_corner_angle_deg = 30.;
    input.params.max_corner_angle_deg = 230.;
    input.params.bevel_cap_deg = 60.;
    input.params.blank_usable = 11.5;
    input.params.depth = args[2].parse().unwrap();
    input.params.randomness = args[3].parse().unwrap();
    input.params.seed = args[4].parse().unwrap();
    if let Some(profile) = args.get(5) {
        input.surface.profile =
            serde_json::from_str(&std::fs::read_to_string(profile).unwrap()).unwrap();
    }
    let surface = Surface::from_profile(&input.surface.profile).unwrap();
    let start = std::time::Instant::now();
    let topology = match rectified::build(&surface, &input.params) {
        Ok(t) => t,
        Err(e) => {
            println!("{}", serde_json::json!({"stage":"C3","error":e}));
            std::process::exit(2)
        }
    };
    let symmetry_policy = rectified_planes::symmetry_policy(&topology, &input.params);
    eprintln!(
        "{}",
        serde_json::json!({"event":"case_start","params":input.params,"rows":topology.rows})
    );
    let assembly = rectified_assembly::solve_assembly_reporting(
        &surface,
        &input.params,
        |phase, attempt| {
            eprintln!(
                "{}",
                serde_json::json!({"event":"attempt","phase":phase,"elapsed":start.elapsed().as_secs_f64(),"report":attempt})
            );
        },
    );
    let first_seconds = start.elapsed().as_secs_f64();
    let determinism = if args.get(6).is_some_and(|a| a == "repeat") {
        let repeated = rectified_assembly::solve_assembly_reporting(
            &surface,
            &input.params,
            |phase, attempt| {
                eprintln!(
                    "{}",
                    serde_json::json!({"event":"repeat_attempt","phase":phase,"report":attempt})
                );
            },
        );
        Some(serde_json::to_vec(&assembly).unwrap() == serde_json::to_vec(&repeated).unwrap())
    } else {
        None
    };
    let accepted = determinism != Some(false)
        && assembly.as_ref().is_ok_and(|a| {
            a.closure.is_ok() && a.solid_errors.is_empty() && a.violations.is_empty()
        });
    println!(
        "{}",
        serde_json::json!({"seconds":first_seconds,"including_repeat_seconds":start.elapsed().as_secs_f64(),"determinism":determinism,"params":input.params,"rows":topology.rows,"symmetry_policy":symmetry_policy,"assembly":assembly,"accepted":accepted,"C6":"HOLD: validation/numbering/nesting, not manufacturing certified"})
    );
    if !accepted {
        std::process::exit(1);
    }
}
