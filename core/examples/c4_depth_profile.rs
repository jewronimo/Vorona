//! Candidate-progress diagnostic of the original uncached auto-depth search.
//! Arguments: profile configured-depth-inches randomness seed.
//! JSONL timings/reports on stdout, candidate-start progress on stderr.
//! Mirrors the original ascending grid and uses the production-equivalent single-fit
//! profiler, deliberately rebuilding C3 to measure the redundant integration cost.
//! External deadline is diagnostic only; native solvers retain deterministic caps.
use vstone_core::{depth, feasibility, model::CoreInput, planes, sidecount, surface::Surface};

fn case(args: &[String]) -> Result<serde_json::Value, String> {
    if args.len() != 4 {
        return Err("expected: cylinder|cone_9_to_6|vase_bulge depth randomness seed".into());
    }
    let mut input: CoreInput = serde_json::from_str(include_str!(
        "../../tests/fixtures/core_input_cylinder.json"
    ))
    .map_err(|e| e.to_string())?;
    input.surface.profile = match args[0].as_str() {
        "cylinder" => input.surface.profile,
        "cone_9_to_6" => serde_json::from_str(include_str!(
            "../../tests/fixtures/profiles/cone_9_to_6.json"
        ))
        .map_err(|e| e.to_string())?,
        "vase_bulge" => serde_json::from_str(include_str!(
            "../../tests/fixtures/profiles/vase_bulge.json"
        ))
        .map_err(|e| e.to_string())?,
        other => return Err(format!("unknown profile {other}")),
    };
    if args[0] != "cylinder" {
        input.params.allowed_sides = vec![5, 6, 7];
    }
    input.params.depth = args[1].parse::<f64>().map_err(|e| e.to_string())?;
    input.params.randomness = args[2].parse::<f64>().map_err(|e| e.to_string())?;
    input.params.seed = args[3].parse::<u64>().map_err(|e| e.to_string())?;
    if !input.params.depth.is_finite() || input.params.depth <= 0. {
        return Err("depth must be positive finite inches".into());
    }
    if !input.params.randomness.is_finite() || !(0. ..=1.).contains(&input.params.randomness) {
        return Err("randomness must be finite in [0, 1]".into());
    }
    let started = std::time::Instant::now();
    // The production selector owns C2/C3, candidate order and C4. In particular,
    // do not use the old diagnostic jitter override or an imported prototype fit.
    let surface = Surface::from_profile(&input.surface.profile).unwrap();
    let r = input
        .surface
        .profile
        .iter()
        .map(|p| p[0])
        .fold(0., f64::max);
    let y = surface.y_range();
    let bound = input
        .params
        .depth
        .max((2. * r).hypot(2. * r).hypot(y[1] - y[0]));
    let steps = ((bound - input.params.depth) / depth::STEP).floor() as u64;
    eprintln!(
        "{}",
        serde_json::json!({"event":"start", "profile":args[0], "bound":bound, "candidates":steps+1})
    );
    for k in 0..=steps {
        let mut p = input.params.clone();
        p.depth += k as f64 * depth::STEP;
        let c3_started = std::time::Instant::now();
        let f = feasibility::check(&surface, &p);
        if !f.feasible {
            return Err("C2 failure".into());
        }
        let t = sidecount::build(&surface, &p)?;
        let c3_seconds = c3_started.elapsed().as_secs_f64();
        eprintln!(
            "{}",
            serde_json::json!({"event":"candidate_start","depth":p.depth,"c3_seconds":c3_seconds,"sites":t.tiles.len(),"elapsed":started.elapsed().as_secs_f64()})
        );
        let m = planes::fit_and_polish(&surface, &t, &p).map_err(|e| e.message)?;
        let passed = m.result.is_ok();
        let report = match &m.result {
            Ok(g) => Some(&g.report),
            Err(e) => e.best.as_ref().map(|g| &g.report),
        };
        println!(
            "{}",
            serde_json::json!({"event":"candidate_done","depth":p.depth,"passed":passed,"c3_seconds":c3_seconds,"fit_seconds":m.fit_seconds,"polish_seconds":m.polish_seconds,"fit_profile":m.fit_profile,"report":report,"elapsed":started.elapsed().as_secs_f64()})
        );
        if passed {
            return Ok(
                serde_json::json!({"event":"selected","depth":p.depth,"elapsed":started.elapsed().as_secs_f64()}),
            );
        }
    }
    Ok(
        serde_json::json!({"event":"exhausted","candidates":steps+1,"elapsed":started.elapsed().as_secs_f64()}),
    )
}

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let result = std::panic::catch_unwind(|| case(&args));
    let record = match result {
        Ok(Ok(record)) => record,
        Ok(Err(error)) => serde_json::json!({"passed": false, "error": error}),
        Err(_) => serde_json::json!({"passed": false, "error": "native case panicked"}),
    };
    println!("{record}");
}
