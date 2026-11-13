//! Spatial diagnostics only; production solver and thresholds are unchanged.
use std::collections::{BTreeMap, BTreeSet};
pub use vstone_core::{model, surface, topology};
use vstone_core::{model::CoreInput, planes, surface::Surface};
#[path = "c4_sizeband_support/seeds.rs"]
mod seeds;
#[path = "c4_sizeband_support/sidecount.rs"]
mod sidecount;
mod feasibility {
    pub fn check(
        s: &crate::surface::Surface,
        p: &crate::model::Params,
    ) -> crate::model::Feasibility {
        let mut f = vstone_core::feasibility::check(s, p);
        if p.allowed_sides == [6] && f.pentagons_needed > 0 {
            let mut control = p.clone();
            control.allowed_sides = vec![5, 6, 7];
            let c = vstone_core::feasibility::check(s, &control);
            if !c.feasible || f.column_changes != 0 || f.heptagons_needed != 0 {
                return f; // No authorization to bypass dislocation/heptagon requirements.
            }
            f.feasible = true;
            f.pentagons_needed = 0;
        }
        f
    }
}
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let profile = &args[0];
    let depth: f64 = args[1].parse().unwrap();
    let mut input: CoreInput = serde_json::from_str(include_str!(
        "../../tests/fixtures/core_input_cylinder.json"
    ))
    .unwrap();
    input.surface.profile = serde_json::from_str(
        &std::fs::read_to_string(format!(
            "{}/../tests/fixtures/profiles/{profile}.json",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap(),
    )
    .unwrap();
    input.params.allowed_sides = if profile == "cone_9_to_6" {
        vec![5, 6, 7]
    } else {
        vec![6]
    };
    input.params.size_variation = args[2].parse().unwrap();
    input.params.randomness = 0.;
    input.params.seed = 1;
    input.params.depth = depth;
    let surface = Surface::from_profile(&input.surface.profile).unwrap();
    // Zero is outside the production input contract. No epsilon substitution.
    let zero_limit = depth == 0. && args.iter().any(|a| a == "--zero-limit");
    if depth <= 0. && !zero_limit {
        println!(
            "{}",
            serde_json::json!({"schema":"c4.spatial/1","profile":profile,"params":input.params,"status":"invalid_input","reason":"production requires depth > 0; no solver executed"})
        );
        return;
    }
    let start = std::time::Instant::now();
    let mut topology_params = input.params.clone();
    if zero_limit {
        topology_params.depth = 0.75;
    }
    let mut topology = match sidecount::build(&surface, &topology_params) {
        Ok(t) => t,
        Err(e) => {
            println!(
                "{}",
                serde_json::json!({"schema":"c4.spatial/1","profile":profile,"params":input.params,"status":"topology_error","error":e,"input":input,"production_feasibility":vstone_core::feasibility::check(&surface,&input.params),"experimental_pentagon_bypass_requested":profile=="vase_bulge"})
            );
            return;
        }
    };
    if zero_limit {
        for seed in &mut topology.tiles {
            seed.depth = 0.;
        }
    }
    let diagnostic = match planes::fit_and_polish(&surface, &topology, &input.params) {
        Ok(d) => d,
        Err(e) => {
            println!(
                "{}",
                serde_json::json!({"schema":"c4.spatial/1","profile":profile,"params":input.params,"status":"fit_error","error":e})
            );
            return;
        }
    };
    let (status, best) = match &diagnostic.result {
        Ok(g) => ("pass", Some(g)),
        Err(e) => ("limits_failed", e.best.as_deref()),
    };
    let mut counts = BTreeMap::new();
    let mut nonaux_counts = BTreeMap::new();
    for (i, t) in topology.tiles.iter().enumerate() {
        *counts.entry(topology.degree(i)).or_insert(0usize) += 1;
        if !t.auxiliary {
            *nonaux_counts.entry(topology.degree(i)).or_insert(0usize) += 1;
        }
    }
    let defects: BTreeSet<_> = (0..topology.tiles.len())
        .filter(|&i| !topology.boundary[i] && matches!(topology.degree(i), 5 | 7))
        .collect();
    let spatial = best.map(|g| {
        let failed: BTreeSet<_> = g.report.violated.iter().map(|(i,_)|*i).collect();
        let yr = surface.y_range();
        let rows: Vec<_> = topology.tiles.iter().enumerate().map(|(i,t)| {
            let y = surface.point(t.u,t.s)[1];
            let h = (y-yr[0])/(yr[1]-yr[0]);
            let near = defects.contains(&i) || topology.neighbors[i].iter().any(|n| defects.contains(n));
            let limits: Vec<_> = g.report.violated.iter().filter(|(tile,_)| *tile==i).map(|(_,s)|s).collect();
            serde_json::json!({"tile":i,"normalized_height":h,"band":((h*4.).floor() as i64).clamp(0,3),"whole":g.whole_tiles[i],"auxiliary":t.auxiliary,"degree":topology.degree(i),"hull":topology.boundary[i],"near_5_7":near,"failed":failed.contains(&i),"limits":limits})
        }).collect();
        let bands: Vec<_> = (0..4).map(|b| {
            let measured: Vec<_> = rows.iter().filter(|r|r["whole"]==true && r["band"]==b).collect();
            serde_json::json!({"band":b,"whole_tiles":measured.len(),"failed_tiles":measured.iter().filter(|r|r["failed"]==true).count(),"violation_entries":measured.iter().map(|r|r["limits"].as_array().unwrap().len()).sum::<usize>()})
        }).collect();
        let proximity: Vec<_> = [true,false].into_iter().map(|near| {
            let measured: Vec<_> = rows.iter().filter(|r|r["whole"]==true && r["near_5_7"]==near).collect();
            serde_json::json!({"near_5_7":near,"whole_tiles":measured.len(),"failed_tiles":measured.iter().filter(|r|r["failed"]==true).count(),"violation_entries":measured.iter().map(|r|r["limits"].as_array().unwrap().len()).sum::<usize>()})
        }).collect();
        serde_json::json!({"bands":bands,"proximity":proximity,"tiles":rows,"report":g.report})
    });
    println!(
        "{}",
        serde_json::json!({"schema":"c4.sizeband/1","input":input,"production_feasibility":vstone_core::feasibility::check(&surface,&input.params),"experimental_pentagon_bypass":profile=="vase_bulge","profile":profile,"params":input.params,"status":status,"seconds":start.elapsed().as_secs_f64(),"zero_limit_experiment":zero_limit,"topology_scaffold_depth":topology_params.depth,"degree_histogram_all":counts,"degree_histogram_nonaux":nonaux_counts,"defect_tiles":defects,"spatial":spatial,"diagnostic":diagnostic})
    );
}

#[test]
fn cone_control_topology_matches_production() {
    let mut input: CoreInput = serde_json::from_str(include_str!(
        "../../tests/fixtures/core_input_cylinder.json"
    ))
    .unwrap();
    input.surface.profile = serde_json::from_str(include_str!(
        "../../tests/fixtures/profiles/cone_9_to_6.json"
    ))
    .unwrap();
    input.params.allowed_sides = vec![5, 6, 7];
    input.params.size_variation = 0.25;
    input.params.seed = 1;
    let s = Surface::from_profile(&input.surface.profile).unwrap();
    assert_eq!(
        sidecount::build(&s, &input.params).unwrap(),
        vstone_core::sidecount::build(&s, &input.params).unwrap()
    );
}
