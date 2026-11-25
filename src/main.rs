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
    println!("--- proof ---");
    print!("{source}");
    if !source.ends_with('\n') {
        println!();
    }
}
