use crate::{
    expression::format_expression, format_comment, format_whitespace, PrintInfo, Rule, SpaceType,
};
use pest::iterators::Pair;

pub fn format_function(function: Pair<Rule>) -> Vec<PrintInfo> {
    let mut print_list = Vec::new();

    for iner in function.into_inner() {
        match iner.as_rule() {
            Rule::keyword => print_list.push(PrintInfo::new(
                format!("{}", iner.as_span().as_str().to_uppercase()),
                SpaceType::None,
            )),
            Rule::variable => print_list.push(PrintInfo::new(
                format!("{}", iner.as_span().as_str()),
                SpaceType::None,
            )),
            Rule::left_parenthesis => print_list.push(PrintInfo::new(
                format!("{}", iner.as_span().as_str()),
                SpaceType::NoSpace,
            )),
            Rule::right_parenthesis => print_list.push(PrintInfo::new(
                format!("{}", iner.as_span().as_str()),
                SpaceType::NoLeftPad,
            )),
            Rule::function_content => print_list.append(&mut format_function_content(iner)),
            une => panic!("{:?},\n{:?} invalid function somehow parsed", une, iner),
        }
    }

    print_list
}

pub fn format_function_declaration(declaration: Pair<Rule>) -> Vec<PrintInfo> {
    let mut print_list = Vec::new();

    for iner in declaration.into_inner() {
        match iner.as_rule() {
            Rule::keyword | Rule::function_keyword | Rule::forward_marker => {
                print_list.push(PrintInfo::new(
                    format!("{}", iner.as_span().as_str().to_uppercase()),
                    SpaceType::None,
                ))
            }
            Rule::variable => print_list.push(PrintInfo::new(
                format!("{}", iner.as_span().as_str()),
                SpaceType::None,
            )),
            Rule::function_content => print_list.append(&mut format_function_content(iner)),
            Rule::WHITESPACE => {
                if let Some(f) = format_whitespace(iner) {
                    print_list.push(f);
                }
            }
            Rule::left_parenthesis => print_list.push(PrintInfo::new(
                format!("{}", iner.as_span().as_str()),
                SpaceType::NoSpace,
            )),
            Rule::right_parenthesis => print_list.push(PrintInfo::new(
                format!("{}", iner.as_span().as_str()),
                SpaceType::NoLeftPad,
            )),
            Rule::COMMENT => print_list.push(PrintInfo::new(format_comment(iner), SpaceType::None)),
            une => panic!(
                "{:?},\n{:?} invalid functional declaration somehow parsed",
                une, iner
            ),
        }
    }

    print_list
}

fn format_function_content(function_content: Pair<Rule>) -> Vec<PrintInfo> {
    let mut print_list = Vec::new();

    for iner in function_content.into_inner() {
        match iner.as_rule() {
            Rule::keyword => print_list.push(PrintInfo::new(
                format!("{}", iner.as_span().as_str().to_uppercase()),
                SpaceType::None,
            )),
            Rule::variable => print_list.push(PrintInfo::new(
                format!("{}", iner.as_span().as_str()),
                SpaceType::None,
            )),
            Rule::comma => print_list.push(PrintInfo::new(
                format!("{}", iner.as_span().as_str()),
                SpaceType::NoLeftPad,
            )),
            Rule::expression => print_list.append(&mut format_expression(iner, false)),
            Rule::WHITESPACE => {
                if let Some(f) = format_whitespace(iner) {
                    print_list.push(f);
                }
            }
            Rule::COMMENT => print_list.push(PrintInfo::new(format_comment(iner), SpaceType::None)),
            une => panic!(
                "{:?},\n{:?} invalid functional content somehow parsed",
                une, iner
            ),
        }
    }

    print_list
}
