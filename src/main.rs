extern crate clap;
extern crate regex;
extern crate ssh2;
extern crate dialoguer;

use std::time::Instant;
use clap::{Command, Arg, ArgAction};

use internal::evaluator::Evaluator;
use internal::lexer::Lexer;
use internal::parser::Parser;

mod internal;

fn main() {
    let cmd = Command::new("reploy")
        .version("0.1.9")
        .arg_required_else_help(true)
        .arg(Arg::new("identity")
                 .short('i')
                 .long("identity")
                 .value_name("KEY FILE")
                 .help("The identity file to use for key-based authentication"),
        )
        .arg(Arg::new("verbose")
                 .short('v')
                 .long("verbose")
                 .action(ArgAction::SetTrue)
                 .help("Enable verbose output"),
        )
        .subcommand(Command::new("run")
            .about("Run the specified recipe")
            .arg(Arg::new("recipe")
                .required(true)
                .help("Load and execute the recipe in the specified file")
            )
        );
    let matches = cmd.get_matches();
    match matches.subcommand() {
        Some(("run", sub_matches)) => {
            let start = Instant::now();
            let recipe = std::fs::read_to_string(sub_matches.get_one::<String>("recipe").unwrap()).expect("Could not read recipe file");
            let lexer = Lexer::new(recipe);
            let mut parser = Parser::new(lexer);
            let mut evaluator = Evaluator::new(parser.parse(), matches.get_flag("verbose"));
            if matches.contains_id("identity") {
                evaluator.set_identity(matches.get_one::<String>("identity").unwrap());
            }
            evaluator.run();
            println!("deployment finished，duration {:?}", Instant::now().duration_since(start));
        }
        _ => unreachable!(),
    }
}
