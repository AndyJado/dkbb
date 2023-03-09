use std::{fs::read_to_string, time::Instant};

use dk_parser::dyna_psr::{Rule, TryParser};
use pest::Parser;

fn main() {
    eg_playground()
}

fn eg_playground() {
    let clock_init = Instant::now();
    let clock_punch = |c: Instant| {
        let punch_parse_file = c.elapsed();
        dbg!(punch_parse_file);
    };
    let file_path = "../source.k";
    let file_str = read_to_string(file_path).expect("file should contain value");
    clock_punch(clock_init);
    let file_node = TryParser::parse(dk_parser::dyna_psr::Rule::file, &file_str)
        .expect("should parse file from str")
        .next()
        .unwrap();
    clock_punch(clock_init);
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
    clock_punch(clock_init);
}
