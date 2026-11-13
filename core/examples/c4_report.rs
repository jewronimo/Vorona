//! Release diagnostic: no successful-limit assertion is silently replaced by a report.
use vstone_core::{model::CoreInput, planes, seeds, sidecount, surface::Surface};
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let path = args
        .get(1)
        .expect("usage: c4_report INPUT.json [randomness seed depth]");
    let mut input: CoreInput =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    if let Some(v) = args.get(2) {
        input.params.randomness = v.parse().unwrap();
    }
    if let Some(v) = args.get(3) {
        input.params.seed = v.parse().unwrap();
    }
    if let Some(v) = args.get(4) {
        input.params.depth = v.parse().unwrap();
    }
    let surface = Surface::from_profile(&input.surface.profile).unwrap();
    let topology = match sidecount::build(&surface, &input.params) {
        Ok(t) => t,
        Err(e) => {
            println!("{}", serde_json::json!({"stage":"C3","error":e}));
            std::process::exit(2)
        }
    };
    let start = std::time::Instant::now();
    let result = planes::optimize(
        &surface,
        &topology,
        &input.params,
        &mut seeds::rng(input.params.seed),
    );
    println!(
        "{}",
        serde_json::json!({"seconds":start.elapsed().as_secs_f64(),"result":result})
    );
}
