extern crate pest;
mod assign;
mod expression;
mod temp_table;

use std::{
    fs::File,
    io::{stdin, BufRead, BufReader},
};

use lazy_static::lazy_static;

use regex::Regex;

use assign::format_assign;
use expression::{format_conditional_expression, format_datatype, format_expression};
use pest::{iterators::Pair, Parser};
use temp_table::format_temp_table;

#[macro_use]
extern crate pest_derive;

#[derive(Debug)]
pub enum IoType {
    FromStdIn,
    FromFile(String),
}
/**
    store special printing instructions related to spacing
    or words require speical spacing
*/
#[derive(PartialEq)]
enum SpaceType {
    None,
    NoSpace, //no space from previous word
    End,
    NewLine,
    ExtraIndent,
    NoRightPad,
    TabPadRight,
}
pub struct PrintInfo {
    line: String,
    spacing_attribute: SpaceType,
}

impl PrintInfo {
    fn new(line: String, spacing_attribute: SpaceType) -> Self {
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

    let sucessful_parse = InputParser::parse(Rule::program, &buf).expect("unsucessful parse");

    let mut indent_level = 0;
    for parse_pair in sucessful_parse {
        for iner in parse_pair.into_inner() {
            match iner.as_rule() {
                Rule::COMMENT => {
                    print!("{}{}", get_tabs(indent_level), format_comment(iner));
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
                Rule::NEWLINE => {
                    println!();
                }
                Rule::EOI => {}
                _ => panic!("unrecongised program {:?}",iner.as_rule()),
            };
        }
    }
}

fn format_statement(statement: Pair<Rule>, indent_level: usize) -> usize {
    let mut print_list = Vec::new();
    let mut next_indent = indent_level;

    /* statements require deeper analysis as printing current words can be context sensitive to later upcoming words
     */
    for iner in statement.into_inner() {
        
        println!("{:?}",iner.as_rule());
        match iner.as_rule() {
            Rule::loop_label => {
                print_list.push(PrintInfo::new(
                    format!("{}", iner.as_span().as_str()),
                    SpaceType::NoSpace,
                ));
            }
            Rule::keyword => {
                print_list.push(PrintInfo::new(
                    format!("{}", &iner.as_span().as_str().to_uppercase()),
                    SpaceType::None,
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
                    SpaceType::NoSpace,
                ));
                next_indent += 1;
            }
            Rule::block_end => {
                print_list.push(PrintInfo::new(
                    format!("{}", &iner.as_span().as_str()).to_uppercase(),
                    SpaceType::End,
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
                SpaceType::End,
            )),
            Rule::datatype => print_list.push(PrintInfo::new(
                format!("{}", &format_datatype(iner)),
                SpaceType::None,
            )),

            _ => eprint!("+++++{:?}+++++++", iner.as_rule()),
        }
    }

    /*actual printing of the words */
    let mut words_iter = print_list.into_iter().enumerate().peekable();
    let mut prev_spacing = None;
    loop {
        let (i, word) = match words_iter.next() {
            Some((j, a_word)) => (j, a_word),
            None => {
                break;
            }
        };
        if i == 0 && word.spacing_attribute != SpaceType::End {
            print!("{}{}", get_tabs(indent_level), word.line.trim());
            continue;
        }
        if word.spacing_attribute == SpaceType::End {
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
            //extra indent specific behaviour handled at newline
            SpaceType::None | SpaceType::ExtraIndent | SpaceType::NoRightPad => {
                match prev_spacing {
                    Some(SpaceType::NoRightPad) | Some(SpaceType::ExtraIndent) => {
                        print!("{}", word.line.trim())
                    }
                    _ => print!(" {}", word.line.trim()),
                }
            }
            SpaceType::TabPadRight => {
                print!("{}\t", word.line.trim());
            }
            SpaceType::NoSpace => {
                print!("{}", word.line.trim())
            }
            SpaceType::End => {
                if words_iter.peek().is_none() {
                    print!("{} ", word.line.trim());
                } else {
                    print!("{}", word.line.trim())
                }
            }
            SpaceType::NewLine => {
                let followup = words_iter.peek();

                match followup {
                    Some((_, peeked_print_info))
                        if peeked_print_info.spacing_attribute == SpaceType::NewLine =>
                    {
                        println!("")
                    }
                    Some((_, peeked_print_info))
                        if peeked_print_info.spacing_attribute == SpaceType::End =>
                    {
                        println!("");
                        if indent_level > 0 {
                            print!("{}", get_tabs(indent_level))
                        }
                    }
                    Some((_, peeked_print_info))
                        if peeked_print_info.spacing_attribute == SpaceType::ExtraIndent =>
                    {
                        println!();

                        print!("{}", get_tabs(indent_level + 2))
                    }
                    _ => {
                        println!("");
                        print!("{}", get_tabs(indent_level + 1))
                    }
                }
            }
        }
        prev_spacing = Some(word.spacing_attribute);
    }

    next_indent
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
                    SpaceType::NewLine,
                ))
            }
            _ => {
                return if white_space.as_str().contains("\n") {
                    Some(PrintInfo::new(
                        format!("{}", white_space.as_str()),
                        SpaceType::NewLine,
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
                    SpaceType::NewLine,
                ))
            } else {
                None
            }
        }
    }
}

fn format_comment(comment: Pair<Rule>) -> String {
    lazy_static! {
        static ref OPEN: Regex = Regex::new(r"/\* ?").unwrap();
        static ref CLOSE: Regex = Regex::new(r" ?\*/").unwrap();
    }

    let open_spaced = OPEN.replace(comment.as_str(), "/* ");

    let closed_spaced = CLOSE.replace(&open_spaced, " */");

    closed_spaced.into_owned()
}
