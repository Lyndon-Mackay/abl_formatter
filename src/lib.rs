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

#[derive(PartialEq)]
enum PrintType {
    None,
    NoSpace, //no space from previous word
    End,
    NewLine,
}
struct PrintInfo {
    line: String,
    criteria: PrintType,
}

impl PrintInfo {
    fn new(line: String, criteria: PrintType) -> Self {
        Self { line, criteria }
    }
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
                    println!(
                        "{}{}",
                        get_tabs(indent_level),
                        iner.clone().as_span().as_str()
                    );
                }
                Rule::statement => indent_level = format_statement(iner.clone(), indent_level),
                Rule::new_line => {
                    println!("")
                }
                Rule::keyword => print!("&&{}&&", iner.as_span().as_str()),
                _ => println!("nothing"),
            };
        }
    }
}

fn format_statement(statement: Pair<Rule>, indent_level: usize) -> usize {
    let mut print_list = Vec::new();
    let mut next_indent = indent_level;
    for iner in statement.into_inner() {
        match iner.as_rule() {
            Rule::loop_label => {
                print_list.push(PrintInfo::new(
                    format!("{}", iner.as_span().as_str()),
                    PrintType::NoSpace,
                ));
            }
            Rule::keyword => {
                print_list.push(PrintInfo::new(
                    format!("{}", &iner.as_span().as_str().to_uppercase()),
                    PrintType::None,
                ));
            }
            Rule::expression => {
                print_list.append(&mut format_expression(iner));
            }
            Rule::block_begin => {
                print_list.push(PrintInfo::new(
                    format!("{}", iner.as_span().as_str()),
                    PrintType::NoSpace,
                ));
                next_indent += 1;
            }
            Rule::block_end => {
                print_list.push(PrintInfo::new(
                    format!("{}", &iner.as_span().as_str()).to_uppercase(),
                    PrintType::End,
                ));
                if next_indent > 0 {
                    next_indent -= 1
                } else {
                    next_indent = 0
                };
            }
            Rule::new_line => {
                print_list.push(PrintInfo::new(format!("\n"), PrintType::NewLine));
            }
            Rule::statement_end => print_list.push(PrintInfo::new(
                format!("{}", &iner.as_span().as_str().to_uppercase()),
                PrintType::End,
            )),
            Rule::datatype => print_list.push(PrintInfo::new(
                format!("{}", &format_datatype(iner)),
                PrintType::None,
            )),

            _ => print!("+++++{:?}+++++++", iner.as_rule()),
        }
    }

    let mut words_iter = print_list.into_iter().enumerate().peekable();
    while words_iter.peek().is_some() {
        let (i, word) = words_iter.next().unwrap();
        if i == 0 && word.criteria != PrintType::End {
            print!("{}{}", get_tabs(indent_level), word.line.trim());
            continue;
        }
        if word.criteria == PrintType::End {
            if indent_level == 0 {
                print!("{}", word.line.trim())
            } else {
                print!("{}{}", get_tabs(indent_level - 1), word.line.trim());
            }
            continue;
        }
        match word.criteria {
            PrintType::None => {
                print!(" {}", word.line.trim())
            }
            PrintType::NoSpace => {
                print!("{}", word.line.trim())
            }
            PrintType::End => {
                print!("{}", word.line.trim())
            }
            PrintType::NewLine => {
                let followup = words_iter.peek();

                match followup {
                    Some((_, peeked_print_info))
                        if peeked_print_info.criteria == PrintType::NewLine =>
                    {
                        println!("$$$$$")
                    }
                    Some((_, peeked_print_info))
                        if peeked_print_info.criteria == PrintType::End =>
                    {
                        println!("");
                        if indent_level > 0 {
                            print!("{}", get_tabs(indent_level))
                        }
                    }
                    _ => {
                        println!("");
                        print!("{}", get_tabs(indent_level + 1))
                    }
                }
            }
        }
    }

    next_indent
}

fn format_expression(expression: Pair<Rule>) -> Vec<PrintInfo> {
    let mut print_list = Vec::new();
    for iner in expression.into_inner() {
        match iner.as_rule() {
            Rule::operator | Rule::keyword => {
                print_list.push(PrintInfo::new(
                    format!("{}", &iner.as_span().as_str().to_uppercase()),
                    PrintType::None,
                ));
            }
            Rule::COMMENT => print_list.push(PrintInfo::new(
                format!("{}", iner.as_span().as_str()),
                PrintType::None,
            )),
            Rule::expression => {
                print_list.append(&mut format_expression(iner));
            }
            Rule::new_line => {
                print_list.push(PrintInfo::new("\n".to_string(), PrintType::NewLine));
            }
            Rule::variable => {
                print_list.push(PrintInfo::new(
                    format!("{}", iner.as_span().as_str()),
                    PrintType::None,
                ));
            }
            Rule::datatype => print_list.push(PrintInfo::new(
                format!("{}", &format_datatype(iner)),
                PrintType::None,
            )),
            _ => print!("{}", iner.as_span().as_str()),
        }
    }
    print_list
}

fn format_datatype(data_type: Pair<Rule>) -> String {
    let mut print_string = String::with_capacity(data_type.as_str().len());
    for iner in data_type.into_inner() {
        match iner.as_rule() {
            Rule::logical => print_string.push_str(&iner.as_span().as_str().to_uppercase()),
            _ => print_string.push_str(iner.as_span().as_str()),
        }
    }
    print_string
}

fn get_tabs(indent_level: usize) -> String {
    let mut tabs = String::with_capacity(indent_level);
    for _ in 0..indent_level {
        tabs.push_str("\t");
    }
    tabs
}
