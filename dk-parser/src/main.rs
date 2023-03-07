use std::fs::read_to_string;

use dk_parser::dyna_psr::{Rule, TryParser};
use pest::Parser;

fn main() {
    eg_playground()
}

fn eg_playground() {
    let file_path = "../source.k";
    let file_str = read_to_string(file_path).expect("file should contain value");
    let file_node = TryParser::parse(dk_parser::dyna_psr::Rule::file, &file_str)
        .expect("should parse file from str")
        .next()
        .unwrap();
    for rule in file_node.into_inner() {
        if rule.as_rule() != Rule::deck {
            continue;
        };
        let Some(keyword) = rule
            .into_inner()
            .next()
            .unwrap()
            .into_inner()
            .next() else {continue;};
        if keyword.as_rule() != Rule::keyword {
            continue;
        };
        println!("{}", keyword.as_str());
    }
}
