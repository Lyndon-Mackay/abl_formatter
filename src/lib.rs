extern crate pest;

use std::{
    fs::File,
    io::{stdin, BufRead, BufReader},
};

use pest::{iterators::Pair, Parser};

#[macro_use]
extern crate pest_derive;

#[derive(Debug)]
pub enum IoType {
    FromStdIn,
    FromFile(String),
}

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct InputParser;

pub fn format_code(input: IoType) {
    //general handling of input for either the console or a file
    let mut input: Box<dyn BufRead> = match input {
        IoType::FromStdIn => Box::new(BufReader::new(stdin())),
        IoType::FromFile(file_name) => {
            let file = File::open(file_name).expect("file not found");

            let reader = BufReader::new(file);

            Box::new(BufReader::new(reader))
        }
    };
    let mut buf = Vec::new();
    input.read_to_end(&mut buf).expect("error reading in input");

    let buf = String::from_utf8_lossy(&buf);
    println!("{:?}", &buf);

    let sucessful_parse = InputParser::parse(Rule::program, &buf).expect("unscessful parse");

    /* println!("{:?}", sucessful_parse);
    println!("finish");*/

    let mut indent_level = 0;
    println!("test{}", get_tabs(indent_level));
    for parse_pair in sucessful_parse {
        for iner in parse_pair.into_inner() {
            match &iner.as_rule() {
                Rule::COMMENT => {
                    println!("{}", iner.clone().as_span().as_str());
                }
                Rule::statement => indent_level = format_statement(iner.clone(), indent_level),
                Rule::new_line => {
                    println!("")
                }
                Rule::keyword => print!("{} ", iner.as_span().as_str()),
                _ => println!("nothing"),
            };
        }
    }
}

fn format_statement(statement: Pair<Rule>, indent_level: usize) -> usize {
    let next_indent = indent_level;
    for iner in statement.into_inner() {
        match iner.as_rule() {
            Rule::loop_label => {
                print!("{}", iner.as_span().as_str());
            }
            Rule::keyword => print!(" {}", iner.as_span().as_str().to_uppercase()),
            Rule::expression => {
                format_expression(iner, indent_level);
            }
            Rule::block_begin => {
                print!(" {}", iner.as_span().as_str());
                return next_indent + 1;
            }
            Rule::block_end => {
                print!(" {}", iner.as_span().as_str());
                if next_indent > 0 {
                    return next_indent - 1;
                }
                return 0;
            }
            Rule::new_line => {
                println!("");
                print!("{}", get_tabs(indent_level))
            }
            Rule::statement_end => {
                print!("{}", iner.as_span().as_str().to_uppercase())
            }
            Rule::datatype => format_datatype(iner),

            _ => print!("+++++{:?}+++++++", iner.as_rule()),
        }
    }
    next_indent
}

fn format_expression(expression: Pair<Rule>, indent_level: usize) {
    for iner in expression.into_inner() {
        match iner.as_rule() {
            Rule::operator | Rule::keyword => {
                print!(" {}", iner.as_span().as_str().to_uppercase())
            }
            Rule::COMMENT => {
                print!("{}", iner.as_span().as_str())
            }
            Rule::expression => {
                format_expression(iner, indent_level);
            }
            Rule::new_line => {
                println!("");
                println!("{}", get_tabs(indent_level));
            }
            Rule::variable => {
                print!(" {}", iner.as_span().as_str());
            }
            Rule::datatype => format_datatype(iner),
            _ => print!(" {}", iner.as_span().as_str()),
        }
    }
}

fn format_datatype(data_type: Pair<Rule>) {
    for iner in data_type.into_inner() {
        match iner.as_rule() {
            Rule::logical => {
                print!(" {}", iner.as_span().as_str().to_uppercase())
            }
            _ => print!("{} ", iner.as_span().as_str()),
        }
    }
}

fn get_tabs(indent_level: usize) -> String {
    let mut tabs = String::with_capacity(indent_level);
    for _ in 0..indent_level {
        tabs.push_str("\t");
    }
    tabs
}
