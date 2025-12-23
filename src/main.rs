use std::env;
use std::fs;
use std::process;

const BOILERPLATE: &str = r#"// SPDX-License-Identifier: LicenseRef-VPL WITH AGPL-3.0-only
pragma solidity 0.8.33;

/**
  @custom:benediction DEVS BENEDICAT ET PROTEGAT CONTRACTVM MEVM
  @title TODO
  @author TODO
  @custom:terry "Is this too much voodoo for the next ten centuries?"

  TODO

  @custom:date TODO.
*/
contract TODO { }
"#;

fn print_help(program: &str) {
    println!("dove is our Solidity formatter and code generator.");
    println!();
    println!("Usage: {} <command> [arguments]", program);
    println!();
    println!("Commands:");
    println!("  preen <file>  Format a Solidity file, preening it into shape.");
    println!("  peck <file>   Extract an interface from a Solidity file.");
    println!("  sing          Output a boilerplate smart contract template.");
    println!("  help          Show this help message.");
}

fn cmd_preen(filename: &str) {
    let source = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file '{}': {}", filename, err);
            process::exit(1);
        }
    };

    match dove::format_solidity(&source) {
        Ok(formatted) => {
            println!("{}", formatted);
        }
        Err(err) => {
            eprintln!("Format error: {}", err);
            process::exit(1);
        }
    }
}

fn cmd_peck(filename: &str) {
    let source = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file '{}': {}", filename, err);
            process::exit(1);
        }
    };

    match dove::peck_solidity(&source) {
        Ok(output) => {
            print!("{}", output);
        }
        Err(err) => {
            eprintln!("Peck error: {}", err);
            process::exit(1);
        }
    }
}

fn cmd_sing() {
    print!("{}", BOILERPLATE);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = &args[0];

    if args.len() < 2 {
        print_help(program);
        process::exit(1);
    }

    match args[1].as_str() {
        "preen" => {
            if args.len() != 3 {
                eprintln!("Usage: {} preen <solidity_file>", program);
                process::exit(1);
            }
            cmd_preen(&args[2]);
        }
        "peck" => {
            if args.len() != 3 {
                eprintln!("Usage: {} peck <solidity_file>", program);
                process::exit(1);
            }
            cmd_peck(&args[2]);
        }
        "sing" => {
            cmd_sing();
        }
        "help" | "--help" | "-h" => {
            print_help(program);
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            eprintln!();
            print_help(program);
            process::exit(1);
        }
    }
}
