use std::{env, process};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os();

    let cargo_executable = match args.nth(1) {
        Some(path) => path,
        None => {
            eprintln!("Expected cargo executable");
            process::exit(1)
        }
    };

    let mut cargo = process::Command::new(cargo_executable);
    cargo.arg("build").args(args);

    let mut verbose = false;
    let mut target = "";

    let mut args = cargo.get_args();
    args.next();

    while let Some(arg) = args.next() {
        let arg = arg.to_str().unwrap();

        if arg == "--verbose" {
            verbose = true;
        } else if let Some(suffix) = arg.strip_prefix("--target") {
            if suffix.is_empty() {
                if let Some(arg) = args.next() {
                    target = arg.to_str().unwrap();
                }
            } else if let Some(suffix) = suffix.strip_prefix('=') {
                target = suffix;
            }
        }
    }

    if target.is_empty() {
        eprintln!("Expected target triple");
        process::exit(1);
    }

    let mut rustflags = env::var("CORROSION_RUSTFLAGS").unwrap_or_else(|_| String::new());

    let languages: Vec<String> = env::var("CORROSION_LINKER_LANGUAGES")
        .unwrap_or_else(|_| "".to_string())
        .trim()
        .split(' ')
        .map(Into::into)
        .collect();

    if !languages.is_empty() {
        rustflags += " -Cdefault-linker-libraries=yes";

        // This loop gets the highest preference link language to use for the linker
        let mut highest_preference: Option<(Option<i32>, &str)> = None;
        for language in &languages {
            highest_preference = Some(
                if let Ok(preference) =
                    env::var(&format!("CORROSION_{}_LINKER_PREFERENCE", language))
                {
                    let preference = preference
                        .parse()
                        .expect("Corrosion internal error: PREFERENCE wrong format");
                    match highest_preference {
                        Some((Some(current), language)) if current > preference => {
                            (Some(current), language)
                        }
                        _ => (Some(preference), language),
                    }
                } else if let Some(p) = highest_preference {
                    p
                } else {
                    (None, language)
                },
            );
        }

        // If a preferred compiler is selected, use it as the linker so that the correct standard, implicit libraries
        // are linked in.
        if let Some((_, language)) = highest_preference {
            if let Ok(compiler) = env::var(&format!("CORROSION_{}_COMPILER", language)) {
                let linker_arg = format!(
                    "CARGO_TARGET_{}_LINKER",
                    target.replace("-", "_").to_uppercase()
                );

                cargo.env(linker_arg, compiler);
            }

            if let Ok(target) = env::var(format!("CORROSION_{}_COMPILER_TARGET", language)) {
                rustflags += format!(" -Clink-args=--target={}", target).as_str();
            }
        }

        let extra_link_args = env::var("CORROSION_LINK_ARGS").unwrap_or_else(|_| "".to_string());
        if !extra_link_args.is_empty() {
            rustflags += format!(" -Clink-args={}", extra_link_args).as_str();
        }

        let rustflags_trimmed = rustflags.trim();
        if verbose {
            println!("Rustflags are: `{}`", rustflags_trimmed);
        }

        cargo.env("RUSTFLAGS", rustflags);
    }

    for var in &["CORROSION_CFLAGS", "CORROSION_CXXFLAGS"] {
        let passed_var = var.strip_prefix("CORROSION_").unwrap();

        match (env::var(var), env::var(passed_var)) {
            (Ok(mut val), Ok(passed_val)) => {
                val += " ";
                val += &passed_val;
                cargo.env(passed_var, val);
            }
            (Ok(val), Err(env::VarError::NotPresent)) => {
                cargo.env(passed_var, val);
            }
            _ => (),
        }

    }

    if verbose {
        println!("Corrosion: {:?}", cargo);
    }

    process::exit(if cargo.status()?.success() { 0 } else { 1 });
}
