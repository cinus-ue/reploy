extern crate clap;
extern crate regex;
extern crate ssh2;

use std::time::Instant;

use clap::{App, Arg, SubCommand};

use internal::evaluator::Evaluator;
use internal::lexer::Lexer;
use internal::parser::Parser;

mod internal;

fn main() {
    let matches = App::new("reploy")
        .version("0.1.5")
        .arg(Arg::with_name("identity")
                 .short("i")
                 .long("identity")
                 .value_name("KEY FILE")
                 .help("The identity file to use for key-based authentication").takes_value(true),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Enable verbose output"),
        )
        .subcommand(SubCommand::with_name("run")
            .about("Run the specified recipe")
            .arg(Arg::with_name("recipe")
                .required(true)
                .help("Load and execute the recipe in the specified file")
            )
        ).get_matches();

    if let Some(arg_matches) = matches.subcommand_matches("run") {
        let start = Instant::now();
        let recipe = std::fs::read_to_string(arg_matches.value_of("recipe").unwrap()).expect("Could not read recipe file");
        let lexer = Lexer::new(&recipe);
        let mut parser = Parser::new(lexer);
        let mut evaluator = Evaluator::new(parser.parse(), matches.is_present("verbose"));
        if matches.is_present("identity") {
            evaluator.set_identity(matches.value_of("identity").unwrap());
        }
        evaluator.run();
        println!("deployment finished，duration {:?}", Instant::now().duration_since(start));
    }
}
