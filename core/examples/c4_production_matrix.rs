//! One native production auto-depth case for C4_production_matrix_runner.py.
//! Arguments: profile configured-depth-inches randomness seed. JSON only on stdout.
use vstone_core::{depth, model::CoreInput};

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
        input.params.allowed_sides = vec![6];
        input.params.size_variation = if args[0] == "cone_9_to_6" { 0.25 } else { 0.30 };
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
    let selection = depth::select(&input);
    let passed = selection.selected_depth.is_some();
    Ok(serde_json::json!({
        "harness": "production-configured-first-v3", "profile": args[0],
        "params": input.params, "passed": passed,
        "total_seconds": started.elapsed().as_secs_f64(), "selection": selection,
        "scope": "native C2/C3/C4 only; C5/C6 clipped-piece certification pending"
    }))
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
