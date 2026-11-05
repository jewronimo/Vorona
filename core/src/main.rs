use std::{env, fs, process::ExitCode};
fn main() -> ExitCode {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: vstone-core <input.json>");
        return ExitCode::FAILURE;
    }
    match fs::read_to_string(&args[1]) {
        Ok(s) => {
            println!("{}", vstone_core::generate_json(&s));
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{}: {e}", args[1]);
            ExitCode::FAILURE
        }
    }
}
