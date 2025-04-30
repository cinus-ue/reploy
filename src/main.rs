extern crate clap;
extern crate dialoguer;
extern crate regex;
extern crate ssh2;

use clap::{Arg, ArgAction, Command};
use std::time::Instant;

use internal::evaluator::Evaluator;
use internal::lexer::Lexer;
use internal::parser::Parser;

mod internal;

fn main() {
    let cmd = Command::new("reploy")
        .version("0.2.0")
        .arg_required_else_help(true)
        .arg(
            Arg::new("identity")
                .short('i')
                .long("identity")
                .value_name("KEY FILE")
                .help("The identity file to use for key-based authentication"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(ArgAction::SetTrue)
                .help("Enable verbose output"),
        )
        .subcommand(
            Command::new("run").about("Run the specified recipe").arg(
                Arg::new("recipe")
                    .required(true)
                    .help("Load and execute the recipe in the specified file"),
            ),
        );
    let matches = cmd.get_matches();
    match matches.subcommand() {
        Some(("run", sub_matches)) => {
            let start = Instant::now();
            let recipe = std::fs::read_to_string(sub_matches.get_one::<String>("recipe").unwrap())
                .expect("Could not read recipe file");
            let lexer = Lexer::new(recipe);
            let mut parser = Parser::new(lexer);
            let parsed_recipe = match parser.parse() {
                Ok(recipe) => recipe,
                Err(e) => {
                    eprintln!("Failed to parse recipe: {}", e);
                    std::process::exit(1);
                }
            };
            let mut evaluator = Evaluator::new(parsed_recipe, matches.get_flag("verbose"));
            if matches.contains_id("identity") {
                evaluator.set_identity(matches.get_one::<String>("identity").unwrap());
            }
            match evaluator.run() {
                Ok(_) => println!(
                    "Deployment finished successfully. Duration: {:?}",
                    Instant::now().duration_since(start)
                ),
                Err(e) => eprintln!("Deployment failed: {}", e),
            }
        }
        _ => unreachable!(),
    }
}
