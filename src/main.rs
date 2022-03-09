mod generator;
mod json_util;
mod parser;
mod tokenizer;

use std::collections::VecDeque;

use clap::Parser;
use generator::Generator;
use tokenizer::Token;

use crate::tokenizer::Tokenizer;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Json  be pretty
    #[clap(short, long)]
    json: String,
}

fn main() {
    let args = Args::parse();

    let tokenizer = Tokenizer::new(args.json);
    let tokens = tokenizer.collect::<Result<VecDeque<Token>, _>>().unwrap();

    let mut parser = parser::Parser::new(tokens);
    let node = parser.parse().unwrap();

    let gen = Generator::new(node);
    println!("{}", gen.generate());
}
