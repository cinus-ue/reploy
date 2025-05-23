extern crate clap;
extern crate dialoguer;
extern crate regex;
extern crate ssh2;

use clap::{Arg, ArgAction, Command};
use std::time::Instant;

use internal::evaluator::Evaluator;
use internal::executor::{Executor, LocalExecutor, SshExecutor};
use internal::lexer::Lexer;
use internal::parser::Parser;

mod internal;

fn main() {
    let cmd = Command::new("reploy")
        .version("0.3.1")
        .arg_required_else_help(true)
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(ArgAction::SetTrue)
                .help("Enable verbose output"),
        )
        .subcommand(
            Command::new("ssh")
                .about("Run the specified recipe over SSH")
                .arg(
                    Arg::new("identity")
                        .short('i')
                        .long("identity")
                        .value_name("KEY FILE")
                        .help("The identity file to use for key-based authentication"),
                )
                .arg(
                    Arg::new("password")
                        .short('p')
                        .long("password")
                        .action(ArgAction::SetTrue)
                        .help("Prompt for password authentication"),
                )
                .arg(
                    Arg::new("recipe")
                        .required(true)
                        .help("Load and execute the recipe in the specified file"),
                ),
        )
        .subcommand(
            Command::new("run")
                .about("Run the specified recipe locally")
                .arg(
                    Arg::new("recipe")
                        .required(true)
                        .help("Load and execute the recipe in the specified file"),
                ),
        );

    let matches = cmd.get_matches();

    let (is_local, sub_matches) = match matches.subcommand() {
        Some(("ssh", m)) => (false, m),
        Some(("run", m)) => (true, m),
        _ => unreachable!(),
    };

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

    let executor: Box<dyn Executor> = if is_local {
        Box::new(LocalExecutor::new())
    } else {
        let mut executor = SshExecutor::new();
        if sub_matches.contains_id("identity") {
            executor.set_identity(sub_matches.get_one::<String>("identity").unwrap());
        }
        if sub_matches.get_flag("password") {
            let password = dialoguer::Password::new()
                .with_prompt("SSH password")
                .interact()
                .unwrap_or_default();
            executor.set_password(&password);
        }
        Box::new(executor)
    };

    let mut evaluator = Evaluator::new(parsed_recipe, matches.get_flag("verbose"), executor);

    match evaluator.run() {
        Ok(_) => println!(
            "Recipe execution completed. Duration: {:?}",
            Instant::now().duration_since(start)
        ),
        Err(e) => eprintln!("Recipe execution failed: {}", e),
    }
}
