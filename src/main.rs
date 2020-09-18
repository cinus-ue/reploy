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
        .version("0.1.1")
        .arg(Arg::with_name("identity")
                 .short("-i")
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
            .about("Run the specified recipe(s)")
            .arg(Arg::with_name("recipe")
                .required(true)
                .help("Load and execute the recipe in the specified file(s)")
            )
        ).get_matches();

    if let Some(_matches) = matches.subcommand_matches("run") {
        let instant = Instant::now();
        let recipe = std::fs::read_to_string(_matches.value_of("recipe").unwrap()).unwrap();
        let lexer = Lexer::new(recipe.as_str());
        let mut parser = Parser::new(lexer);
        let mut evaluator = Evaluator::new(parser.parse(), matches.is_present("verbose"));
        if matches.is_present("identity") {
            evaluator.set_identity(matches.value_of("identity").unwrap());
        }
        evaluator.run();
        println!("deployment finished，duration {:?}", Instant::now().duration_since(instant));
    }
}
