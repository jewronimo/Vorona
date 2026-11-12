//! Non-fail-fast production C4 matrix. One complete machine-readable JSON record per case.
use vstone_core::{model::CoreInput, planes, sidecount, surface::Surface};
fn main() {
    let base: CoreInput = serde_json::from_str(include_str!(
        "../../tests/fixtures/core_input_cylinder.json"
    ))
    .unwrap();
    let args: Vec<_> = std::env::args().skip(1).collect();
    let selected_profile = args.first().cloned();
    let cap: f64 = args.get(4).map(|v| v.parse().unwrap()).unwrap_or(0.1);
    for profile in ["cylinder", "cone_9_to_6", "vase_bulge"] {
        if selected_profile
            .as_deref()
            .is_some_and(|name| name != profile)
        {
            continue;
        }
        for depth in [1.5, 1., 2.] {
            let cases: Vec<(f64, u64)> = if depth == 1.5 {
                [0., 0.5, 1.]
                    .into_iter()
                    .flat_map(|r| (1..=10).map(move |s| (r, s)))
                    .collect()
            } else {
                vec![(0., 1)]
            };
            for (randomness, seed) in cases {
                if args.len() >= 4
                    && (depth != args[1].parse::<f64>().unwrap()
                        || randomness != args[2].parse::<f64>().unwrap()
                        || seed != args[3].parse::<u64>().unwrap())
                {
                    continue;
                }
                let started = std::time::Instant::now();
                let result = std::panic::catch_unwind(|| {
                    let mut input = base.clone();
                    input.params.target_size = 6.5;
                    input.params.size_variation = 0.15;
                    input.params.depth = depth;
                    input.params.randomness = randomness;
                    input.params.seed = seed;
                    input.params.max_corner_angle_deg = 230.;
                    if profile != "cylinder" {
                        input.surface.profile = serde_json::from_str(
                            &std::fs::read_to_string(format!(
                                "{}/../tests/fixtures/profiles/{profile}.json",
                                env!("CARGO_MANIFEST_DIR")
                            ))
                            .unwrap(),
                        )
                        .unwrap();
                        input.params.allowed_sides = vec![5, 6, 7];
                    }
                    let surface = Surface::from_profile(&input.surface.profile)
                        .map_err(|e| format!("{e:?}"))?;
                    let topology_start = std::time::Instant::now();
                    let topology = sidecount::build_with_jitter(&surface, &input.params, cap)
                        .map_err(|e| format!("{e:?}"))?;
                    let topology_seconds = topology_start.elapsed().as_secs_f64();
                    let diagnostic = planes::fit_and_polish(&surface, &topology, &input.params)
                        .map_err(|e| format!("{e:?}"))?;
                    Ok::<_, String>(
                        serde_json::json!({"params":input.params,"sites":topology.tiles.len(),"auxiliary_sites":topology.tiles.iter().filter(|s|s.auxiliary).count(),"open_hull_sites":topology.boundary.iter().filter(|&&v|v).count(),"topology_seconds":topology_seconds,"fit_seconds":diagnostic.fit_seconds,"polish_seconds":diagnostic.polish_seconds,"optimizer_total_seconds":diagnostic.total_seconds,"fit_profile":diagnostic.fit_profile,"fit":diagnostic.fit,"result":diagnostic.result}),
                    )
                });
                let result = match result {
                    Ok(Ok(v)) => v,
                    Ok(Err(e)) => serde_json::json!({"error":e}),
                    Err(_) => serde_json::json!({"error":"case panicked; matrix continues"}),
                };
                println!(
                    "{}",
                    serde_json::json!({"harness":"single-fit-v1","jitter_cap":cap,"profile":profile,"depth":depth,"randomness":randomness,"seed":seed,"acceptance":depth==1.5,"total_seconds":started.elapsed().as_secs_f64(),"measurement":result})
                );
            }
        }
    }
}
