use crate::{format_whitespace, PrintInfo, PrintType, Rule};
use pest::iterators::Pair;

#[derive(PartialEq)]
enum BracketFormatting {
    TwoOpening(bool, bool),
    None,
}

pub fn format_conditional_expression(conditional_expression: Pair<Rule>) -> Vec<PrintInfo> {
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
            Rule::COMMENT => print_list.push(PrintInfo::new(
                format!("{}", iner.as_span().as_str()),
                PrintType::None,
            )),
            une => panic!(
                "{:?},\n{:?} invalid conditional expression somehow parsed",
                une, iner
            ),
        }
    }

    print_list
}

pub fn format_expression(expression: Pair<Rule>, conditional: bool) -> Vec<PrintInfo> {
    let two_opening_brackets = match expression
        .clone()
        .into_inner()
        .filter_map(|x| match x.as_rule() {
            Rule::COMMENT => None,
            Rule::WHITESPACE if !x.as_str().contains("\n") => None,
            Rule::expression => Some(x.into_inner().next()?.as_rule()),
            _ => Some(x.as_rule()),
        })
        .take(3)
        .collect::<Vec<Rule>>()
        .as_slice()
    {
        [Rule::left_parenthesis, Rule::WHITESPACE, Rule::left_parenthesis] => true,
        _ => false,
    };
    if two_opening_brackets {
        inner_format_expression(
            expression,
            conditional,
            &mut BracketFormatting::TwoOpening(true, true),
            &mut 0,
        )
    } else {
        inner_format_expression(
            expression,
            conditional,
            &mut BracketFormatting::None,
            &mut 0,
        )
    }
}

fn inner_format_expression(
    expression: Pair<Rule>,
    conditional: bool,
    brackets: &mut BracketFormatting,
    unclosed_left_count: &mut usize,
) -> Vec<PrintInfo> {
    let mut print_list = Vec::new();

    let mut iter = expression.into_inner();
    loop {
        let iner = match iter.next() {
            Some(x) => x,
            None => break print_list,
        };
        match iner.as_rule() {
            Rule::keyword => {
                print_list.push(PrintInfo::new(
                    format!("{}", &iner.as_span().as_str().to_uppercase()),
                    PrintType::None,
                ));
            }
            Rule::operator => {
                //check to add tab before left parenthesis if an outermost logical
                let pr_type = if *unclosed_left_count == 1
                    && *brackets != BracketFormatting::None
                    && iter
                        .clone()
                        .filter_map(|x| match x.as_rule() {
                            Rule::WHITESPACE | Rule::COMMENT => None,
                            Rule::expression => Some(x.into_inner().next()?.as_rule()),
                            x => Some(x),
                        })
                        .next()
                        == Some(Rule::left_parenthesis)
                {
                    PrintType::TabPadRight
                } else {
                    PrintType::None
                };

                print_list.push(PrintInfo::new(format_operator(iner, conditional), pr_type))
            }
            Rule::COMMENT => print_list.push(PrintInfo::new(
                format!("{}", iner.as_span().as_str()),
                PrintType::None,
            )),
            Rule::expression => {
                print_list.append(&mut inner_format_expression(
                    iner,
                    conditional,
                    brackets,
                    unclosed_left_count,
                ));
            }
            Rule::WHITESPACE => {
                if *brackets == BracketFormatting::None {
                    if let Some(f) = format_whitespace(iner) {
                        print_list.push(f);
                    }
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
            Rule::function => print_list.append(&mut format_function(iner)),
            Rule::right_parenthesis => {
                *unclosed_left_count -= 1;
                print_list.push(PrintInfo::new(
                    format!("{}", iner.as_span().as_str()),
                    PrintType::NoSpace,
                ));
                if *unclosed_left_count == 1 && *brackets != BracketFormatting::None {
                    print_list.push(PrintInfo::new(format!("\n"), PrintType::NewLine));
                }
            }
            Rule::left_parenthesis => {
                *unclosed_left_count += 1;

                match brackets {
                    BracketFormatting::TwoOpening(f, _) if *f => {
                        print_list.push(PrintInfo::new(
                            format!("{}", iner.as_span().as_str()),
                            PrintType::None,
                        ));
                        print_list.push(PrintInfo::new(format!("\n"), PrintType::NewLine));
                        *brackets = BracketFormatting::TwoOpening(false, true);
                    }
                    BracketFormatting::TwoOpening(_, f2) if *f2 => {
                        print_list.push(PrintInfo::new(
                            format!("{}", iner.as_span().as_str()),
                            PrintType::ExtraIndent,
                        ));
                        *brackets = BracketFormatting::TwoOpening(false, false);
                    }
                    BracketFormatting::None if *unclosed_left_count == 1 => {
                        //if next value is left parenthesis change to special formatting
                        if iter
                            .clone()
                            .filter_map(|x| match x.as_rule() {
                                Rule::WHITESPACE | Rule::COMMENT => None,
                                Rule::expression => Some(x.into_inner().next()?.as_rule()),
                                x => Some(x),
                            })
                            .next()
                            == Some(Rule::left_parenthesis)
                        {
                            print_list.push(PrintInfo::new(format!("\n"), PrintType::NewLine));
                            print_list.push(PrintInfo::new(
                                format!("{}", iner.as_span().as_str()),
                                PrintType::None,
                            ));
                            print_list.push(PrintInfo::new(format!("\n"), PrintType::NewLine));
                            *brackets = BracketFormatting::TwoOpening(false, true);
                        } else {
                            print_list.push(PrintInfo::new(
                                format!("{}", iner.as_span().as_str()),
                                PrintType::NoRightPad,
                            ));
                        }
                    }
                    _ => {
                        print_list.push(PrintInfo::new(
                            format!("{}", iner.as_span().as_str()),
                            PrintType::NoRightPad,
                        ));
                    }
                }
            }
            une => panic!("{:?}{}", une, iner.as_span().as_str()),
        }
    }
}

pub fn format_datatype(data_type: Pair<Rule>) -> String {
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

fn format_function(function: Pair<Rule>) -> Vec<PrintInfo> {
    let mut print_list = Vec::new();

    for iner in function.into_inner() {
        match iner.as_rule() {
            Rule::keyword => print_list.push(PrintInfo::new(
                format!("{}", iner.as_span().as_str().to_uppercase()),
                PrintType::None,
            )),
            Rule::variable => print_list.push(PrintInfo::new(
                format!("{}", iner.as_span().as_str()),
                PrintType::None,
            )),
            Rule::left_parenthesis | Rule::right_parenthesis => print_list.push(PrintInfo::new(
                format!("{}", iner.as_span().as_str()),
                PrintType::NoSpace,
            )),
            Rule::function_content => print_list.append(&mut format_function_content(iner)),
            une => panic!("{:?},\n{:?} invalid function somehow parsed", une, iner),
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
                PrintType::None,
            )),
            Rule::variable => print_list.push(PrintInfo::new(
                format!("{}", iner.as_span().as_str()),
                PrintType::None,
            )),
            Rule::comma => print_list.push(PrintInfo::new(
                format!("{}", iner.as_span().as_str()),
                PrintType::NoSpace,
            )),
            Rule::expression => print_list.append(&mut format_expression(iner, false)),
            Rule::WHITESPACE => {
                if let Some(f) = format_whitespace(iner) {
                    print_list.push(f);
                }
            }
            Rule::COMMENT => print_list.push(PrintInfo::new(
                format!("{}", iner.as_span().as_str()),
                PrintType::None,
            )),
            une => panic!(
                "{:?},\n{:?} invalid functional content somehow parsed",
                une, iner
            ),
        }
    }

    print_list
}
