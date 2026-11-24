//! A06 native C3-C5 physical-band diagnostics; C6/C7 manufacturing gates held.
use vstone_core::{
    model::{CoreInput, Pattern},
    rectified, rectified_assembly,
    surface::Surface,
};
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let mut input: CoreInput =
        serde_json::from_str(&std::fs::read_to_string(&args[1]).unwrap()).unwrap();
    if input.params.pattern != Pattern::VoronoiTriangles {
        input.params.min_triangle_edge = 2.0;
        input.params.max_polygon_edge = None;
    }
    input.params.pattern = Pattern::VoronoiTriangles;
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
    if let Some(mode) = args.get(7) {
        input.params.row_height =
            serde_json::from_value(serde_json::Value::String(mode.clone())).unwrap();
    }
    let surface = Surface::from_profile(&input.surface.profile)
        .unwrap()
        .for_pattern(vstone_core::model::Pattern::VoronoiTriangles);
    let start = std::time::Instant::now();
    let topology = match rectified::build(&surface, &input.params) {
        Ok(t) => t,
        Err(e) => {
            println!("{}", serde_json::json!({"stage":"C3","error":e}));
            std::process::exit(2)
        }
    };
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
    // Use the selected phase's final measured geometry, not pre-solve classification.
    // An absent final geometry must not be presented as a measured pass.
    let symmetry_policy = assembly.as_ref().ok().map(|a| &a.symmetry_rows);
    let actual_largest_side = assembly.as_ref().ok().map(|a| {
        a.pieces
            .iter()
            .flat_map(|p| &p.edges)
            .map(|e| e.length)
            .fold(0_f64, f64::max)
    });
    let actual_shortest_side = assembly.as_ref().ok().and_then(|a| {
        a.pieces
            .iter()
            .flat_map(|p| &p.edges)
            .map(|e| e.length)
            .min_by(f64::total_cmp)
    });
    let actual_edge_shortfall = assembly.as_ref().ok().map(|a| {
        a.pieces
            .iter()
            .filter(|p| !p.partial)
            .flat_map(|p| &p.edges)
            .map(|e| (input.params.min_triangle_edge - e.length).max(0.))
            .fold(0_f64, f64::max)
    });
    let edge_range = |kind: &str| {
        let values: Vec<f64> = assembly
            .as_ref()
            .ok()
            .into_iter()
            .flat_map(|a| &a.pieces)
            .flat_map(|p| &p.edges)
            .filter(|e| e.kind == kind)
            .map(|e| e.length)
            .collect();
        (
            values.iter().copied().min_by(f64::total_cmp),
            values.iter().copied().max_by(f64::total_cmp),
        )
    };
    let joint_edges = edge_range("joint");
    let saw_cut_edges = edge_range("rim_level");
    let largest_piece_width = assembly.as_ref().ok().map(|a| {
        a.pieces
            .iter()
            .map(|p| p.bounding_square)
            .fold(0_f64, f64::max)
    });
    let accepted = determinism != Some(false)
        && assembly.as_ref().is_ok_and(|a| {
            a.closure.is_ok()
                && a.solid_errors.is_empty()
                && a.violations.is_empty()
                && a.symmetry_rows
                    .iter()
                    .all(|r| !r.required || r.passed == Some(true))
        });
    println!(
        "{}",
        serde_json::json!({"shortest_joint_edge":joint_edges.0,"largest_joint_edge":joint_edges.1,"shortest_saw_cut_edge":saw_cut_edges.0,"largest_saw_cut_edge":saw_cut_edges.1,"largest_piece_width":largest_piece_width,"blank_usable":input.params.blank_usable,"actual_shortest_side":actual_shortest_side,"actual_edge_shortfall":actual_edge_shortfall,"actual_largest_side":actual_largest_side,"seconds":first_seconds,"including_repeat_seconds":start.elapsed().as_secs_f64(),"determinism":determinism,"params":input.params,"rows":topology.rows,"symmetry_policy":symmetry_policy,"assembly":assembly,"accepted":accepted,"C6":"HOLD: validation/numbering/nesting, not manufacturing certified"})
    );
    if !accepted {
        std::process::exit(1);
    }
}
