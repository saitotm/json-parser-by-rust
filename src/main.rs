// TODO: add option to specify indent (space or tab, count)
// TODO: add cui usage to README.md
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

    #[clap(long, short, default_value_t = 4)]
    /// indent size
    n: usize,
}

fn pretty_json(json: String, indent_size: usize) -> Result<String, String> {
    let tokenizer = Tokenizer::new(json);
    let tokens = tokenizer.collect::<Result<VecDeque<Token>, _>>()?;

    let mut parser = parser::Parser::new(tokens);
    let node = parser.parse()?;

    let gen = Generator::new(node, indent_size);
    Ok(gen.generate())
}

fn main() {
    let args = Args::parse();
    match pretty_json(args.json_text, args.n) {
        Ok(json) => println!("{}", json),
        Err(err) => eprintln!("Error: {}", err),
    }
}
