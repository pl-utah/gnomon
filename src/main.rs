use std::{env, fs, process};

fn main() {
    let Some(path) = env::args_os().nth(1) else {
        eprintln!("usage: gnomon <proof-file>");
        process::exit(2);
    };
    let source = fs::read_to_string(&path).unwrap_or_else(|error| {
        eprintln!("could not read proof file: {error}");
        process::exit(2);
    });
    print!("{source}");
    if !source.ends_with('\n') {
        println!();
    }
    match gnomon::script::run(&source) {
        Ok(()) => println!("proof is correct"),
        Err(error) => {
            println!("proof is not correct");
            eprintln!("{error}");
            process::exit(1);
        }
    }
}
