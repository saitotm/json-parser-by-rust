// Todo: add option to specify indent (space or tab, count)
mod generator;
mod json_util;
mod parser;
mod tokenizer;

use std::collections::VecDeque;

use clap::Parser;
use generator::Generator;
use tokenizer::Token;

use crate::tokenizer::Tokenizer;

/// Simple lint for JSON text
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// JSON text which you want to lint
    json_text: String,
}

fn pretty_json(json: String) -> Result<String, String> {
    let tokenizer = Tokenizer::new(json);
    let tokens = tokenizer.collect::<Result<VecDeque<Token>, _>>()?;

    let mut parser = parser::Parser::new(tokens);
    let node = parser.parse()?;

    let gen = Generator::new(node);
    Ok(gen.generate())
}

fn main() {
    let args = Args::parse();
    match pretty_json(args.json_text) {
        Ok(json) => println!("{}", json),
        Err(err) => eprintln!("Error: {}", err),
    }
}
