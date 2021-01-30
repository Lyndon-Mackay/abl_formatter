extern crate pest;
mod assign;
mod temp_table;

use std::{
    fs::File,
    io::{stdin, BufRead, BufReader},
};

use assign::format_assign;
use pest::{iterators::Pair, Parser};
use temp_table::format_temp_table;

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
pub struct PrintInfo {
    line: String,
    spacing_attribute: PrintType,
}

impl PrintInfo {
    fn new(line: String, spacing_attribute: PrintType) -> Self {
        Self {
            line,
            spacing_attribute,
        }
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
    //    println!("{:?}", &buf);

    let sucessful_parse = InputParser::parse(Rule::program, &buf).expect("unscessful parse");

    /* println!("{:?}", sucessful_parse);
    println!("finish");*/

    let mut indent_level = 0;
    for parse_pair in sucessful_parse {
        for iner in parse_pair.into_inner() {
            match &iner.as_rule() {
                Rule::COMMENT => {
                    print!(
                        "{}{}",
                        get_tabs(indent_level),
                        iner.clone().as_span().as_str()
                    );
                }
                Rule::statement => indent_level = format_statement(iner.clone(), indent_level),
                Rule::WHITESPACE => {
                    if iner.as_str().contains("\n") {
                        println!();
                    } else if let Some(potential_newline) = iner.into_inner().next() {
                        if let Rule::NEWLINE = potential_newline.as_rule() {
                            println!()
                        }
                    }
                }
                Rule::keyword => print!("{}", iner.as_span().as_str().to_uppercase()),
                Rule::include => {
                    println!("{}{}", get_tabs(indent_level), iner.as_span().as_str())
                }
                Rule::EOI => {}
                _ => panic!("unrecongised program"),
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
            Rule::conditional_expression => {
                print_list.append(&mut format_conditional_expression(iner))
            }
            Rule::expression => {
                print_list.append(&mut format_expression(iner, false));
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
            Rule::define_temp_table => {
                print_list.append(&mut format_temp_table(iner));
            }
            Rule::assign_statement => {
                print_list.append(&mut format_assign(iner));
            }
            Rule::WHITESPACE => {
                if let Some(f) = format_whitespace(iner) {
                    print_list.push(f);
                }
            }
            Rule::NEWLINE => {
                println!();
            }
            Rule::statement_end => print_list.push(PrintInfo::new(
                format!("{}", &iner.as_span().as_str().to_uppercase()),
                PrintType::End,
            )),
            Rule::datatype => print_list.push(PrintInfo::new(
                format!("{}", &format_datatype(iner)),
                PrintType::None,
            )),

            _ => eprint!("+++++{:?}+++++++", iner.as_rule()),
        }
    }

    let mut words_iter = print_list.into_iter().enumerate().peekable();
    loop {
        let (i, word) = match words_iter.next() {
            Some((j, a_word)) => (j, a_word),
            None => {
                break;
            }
        };
        if i == 0 && word.spacing_attribute != PrintType::End {
            print!("{}{}", get_tabs(indent_level), word.line.trim());
            continue;
        }
        if word.spacing_attribute == PrintType::End {
            if words_iter.peek().is_none() {
                if indent_level == 0 {
                    print!("{} ", word.line.trim())
                } else {
                    print!("{}{} ", get_tabs(indent_level - 1), word.line.trim());
                }
            } else {
                if indent_level == 0 {
                    print!("{}", word.line.trim())
                } else {
                    print!("{}{}", get_tabs(indent_level - 1), word.line.trim());
                }
            }
            continue;
        }
        match word.spacing_attribute {
            PrintType::None => {
                print!(" {}", word.line.trim())
            }
            PrintType::NoSpace => {
                print!("{}", word.line.trim())
            }
            PrintType::End => {
                if words_iter.peek().is_none() {
                    print!("{} ", word.line.trim());
                } else {
                    print!("{}", word.line.trim())
                }
            }
            PrintType::NewLine => {
                let followup = words_iter.peek();

                match followup {
                    Some((_, peeked_print_info))
                        if peeked_print_info.spacing_attribute == PrintType::NewLine =>
                    {
                        println!("")
                    }
                    Some((_, peeked_print_info))
                        if peeked_print_info.spacing_attribute == PrintType::End =>
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

fn format_conditional_expression(conditional_expression: Pair<Rule>) -> Vec<PrintInfo> {
    let mut print_list = Vec::new();
    for iner in conditional_expression.into_inner() {
        match iner.as_rule() {
            Rule::conditional_expression_pred => {
                print_list.push(PrintInfo::new(
                    format!("{}", &iner.as_span().as_str().to_uppercase()),
                    PrintType::None,
                ));
            }
            Rule::expression => print_list.append(&mut format_expression(iner, true)),
            Rule::WHITESPACE => {
                if let Some(f) = format_whitespace(iner) {
                    print_list.push(f);
                }
            }
            une => panic!(
                "{:?},\n{:?} invalid conditional expression somehow parsed",
                une, iner
            ),
        }
    }
    print_list
}

fn format_expression(expression: Pair<Rule>, conditional: bool) -> Vec<PrintInfo> {
    let mut print_list = Vec::new();
    for iner in expression.into_inner() {
        match iner.as_rule() {
            Rule::keyword => {
                print_list.push(PrintInfo::new(
                    format!("{}", &iner.as_span().as_str().to_uppercase()),
                    PrintType::None,
                ));
            }
            Rule::operator => print_list.push(PrintInfo::new(
                format_operator(iner, conditional),
                PrintType::None,
            )),
            Rule::COMMENT => print_list.push(PrintInfo::new(
                format!("{}", iner.as_span().as_str()),
                PrintType::None,
            )),
            Rule::expression => {
                print_list.append(&mut format_expression(iner, conditional));
            }
            Rule::WHITESPACE => {
                if let Some(f) = format_whitespace(iner) {
                    print_list.push(f);
                }
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

fn format_operator(operator: Pair<Rule>, conditional: bool) -> String {
    match operator.as_span().as_str().trim() {
        "<" => format!("LT"),
        "<=" => format!("LE"),
        ">" => format!("GT"),
        ">=" => format!("GE"),
        "=" if conditional => format!("EQ"),
        misc => format!("{}", misc.to_uppercase()),
    }
}

fn get_tabs(indent_level: usize) -> String {
    let mut tabs = String::with_capacity(indent_level);
    for _ in 0..indent_level {
        tabs.push_str("\t");
    }
    tabs
}

fn format_whitespace(white_space: Pair<Rule>) -> Option<PrintInfo> {
    match white_space.clone().into_inner().next() {
        Some(iner) => match iner.as_rule() {
            Rule::NEWLINE => {
                return Some(PrintInfo::new(
                    format!("{}", iner.as_span().as_str()),
                    PrintType::NewLine,
                ))
            }
            _ => {
                return if white_space.as_str().contains("\n") {
                    Some(PrintInfo::new(
                        format!("{}", white_space.as_str()),
                        PrintType::NewLine,
                    ))
                } else {
                    None
                }
            }
        },
        None => {
            if white_space.as_str().contains("\n") {
                Some(PrintInfo::new(
                    format!("{}", white_space.as_str()),
                    PrintType::NewLine,
                ))
            } else {
                None
            }
        }
    }
}
