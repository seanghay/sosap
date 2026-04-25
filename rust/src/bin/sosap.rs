use std::process::ExitCode;

use sosap::{Model, PhoneticizeOptions};

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let model_path = match args.next() {
        Some(p) => p,
        None => {
            eprintln!("usage: sosap <model.fst> <word> [--nbest N] [--beam B] [--threshold T] [--pmass P] [--accumulate]");
            return ExitCode::from(2);
        }
    };
    let word = match args.next() {
        Some(w) => w,
        None => {
            eprintln!("missing word");
            return ExitCode::from(2);
        }
    };

    let mut opts = PhoneticizeOptions::default();
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--nbest" => opts.nbest = args.next().and_then(|s| s.parse().ok()).unwrap_or(opts.nbest),
            "--beam" => opts.beam = args.next().and_then(|s| s.parse().ok()).unwrap_or(opts.beam),
            "--threshold" => {
                opts.threshold = args.next().and_then(|s| s.parse().ok()).unwrap_or(opts.threshold)
            }
            "--pmass" => opts.pmass = args.next().and_then(|s| s.parse().ok()).unwrap_or(opts.pmass),
            "--accumulate" => opts.accumulate = true,
            "--write-fsts" => opts.write_fsts = true,
            other => {
                eprintln!("unknown flag: {other}");
                return ExitCode::from(2);
            }
        }
    }

    let model = match Model::open(&model_path, "") {
        Ok(m) => m,
        Err(e) => {
            eprintln!("failed to load model {model_path}: {e}");
            return ExitCode::from(1);
        }
    };

    let paths = model.phoneticize(&word, &opts);
    for path in &paths {
        let phonemes: Vec<String> = path
            .uniques
            .iter()
            .filter_map(|&id| model.find_osym_name(id).map(str::to_string))
            .collect();
        println!("{:.4}\t{}", path.path_weight, phonemes.join(" "));
    }
    ExitCode::SUCCESS
}
