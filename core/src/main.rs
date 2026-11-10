use std::{env, fs, process::ExitCode};
fn main() -> ExitCode {
    let args: Vec<_> = env::args().collect();
    let only = args.get(1).is_some_and(|arg| arg == "--feasibility-only");
    if args.len() != if only { 3 } else { 2 } {
        eprintln!("usage: vstone-core [--feasibility-only] <input.json>");
        return ExitCode::FAILURE;
    }
    let path = &args[if only { 2 } else { 1 }];
    match fs::read_to_string(path) {
        Ok(s) => {
            println!(
                "{}",
                if only {
                    vstone_core::feasibility_only_json(&s)
                } else {
                    vstone_core::generate_json(&s)
                }
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{path}: {e}");
            ExitCode::FAILURE
        }
    }
}
