use crate::{
    format_comment, format_whitespace, function::format_function, PrintInfo, Rule, SpaceType,
};
use pest::iterators::Pair;

#[derive(PartialEq)]
enum BracketFormatting {
    TwoOpening(bool, bool),
    None,
}

pub fn format_by_expression(by_expression: Pair<Rule>) -> Vec<PrintInfo> {
    let mut print_list = Vec::new();

    for iner in by_expression.into_inner() {
        match iner.as_rule() {
            Rule::by_keyword => print_list.push(PrintInfo::new(
                format!("{}", &iner.as_span().as_str().to_uppercase()),
                SpaceType::None,
            )),
            Rule::expression => print_list.append(&mut format_expression(iner, false)),

            Rule::WHITESPACE => {
                if let Some(f) = format_whitespace(iner) {
                    print_list.push(f);
                }
            }
            Rule::COMMENT => print_list.push(PrintInfo::new(
                format!("{}", iner.as_span().as_str()),
                SpaceType::None,
            )),
            une => panic!(
                "{:?},\n{:?} invalid by expression somehow parsed",
                une, iner
            ),
        }
    }
    print_list
}

pub fn format_conditional_expression(conditional_expression: Pair<Rule>) -> Vec<PrintInfo> {
    let mut print_list = Vec::new();

    for iner in conditional_expression.into_inner() {
        match iner.as_rule() {
            Rule::conditional_expression_pred => {
                print_list.push(PrintInfo::new(
                    format!("{}", &iner.as_span().as_str().to_uppercase()),
                    SpaceType::None,
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
                SpaceType::None,
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
        //println!("{:?}", iner);
        match iner.as_rule() {
            Rule::keyword
            | Rule::not_keyword
            | Rule::logical
            | Rule::properties
            | Rule::temptable_keyword => {
                print_list.push(PrintInfo::new(
                    format!("{}", &iner.as_span().as_str().to_uppercase()),
                    SpaceType::None,
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
                    SpaceType::TabPadRight
                } else {
                    SpaceType::None
                };

                print_list.push(PrintInfo::new(format_operator(iner, conditional), pr_type))
            }
            Rule::COMMENT => print_list.push(PrintInfo::new(
                format!("{}", format_comment(iner)),
                SpaceType::None,
            )),
            Rule::expression => {
                print_list.append(&mut inner_format_expression(
                    iner,
                    conditional,
                    brackets,
                    unclosed_left_count,
                ));
            }
            Rule::accumulate => {
                print_list.append(&mut format_accumulate(iner))
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
                    SpaceType::None,
                ));
            }
            Rule::datatype => print_list.append(&mut format_datatype(iner)),
            Rule::function => print_list.append(&mut format_function(iner)),
            Rule::right_parenthesis => {
                *unclosed_left_count -= 1;
                print_list.push(PrintInfo::new(
                    format!("{}", iner.as_span().as_str()),
                    SpaceType::NoLeftPad,
                ));
                if *unclosed_left_count == 1 && *brackets != BracketFormatting::None {
                    print_list.push(PrintInfo::new(format!("\n"), SpaceType::NewLine));
                }
            }

            Rule::left_parenthesis => {
                *unclosed_left_count += 1;

                match brackets {
                    BracketFormatting::TwoOpening(f, _) if *f => {
                        print_list.push(PrintInfo::new(
                            format!("{}", iner.as_span().as_str()),
                            SpaceType::None,
                        ));
                        print_list.push(PrintInfo::new(format!("\n"), SpaceType::NewLine));
                        *brackets = BracketFormatting::TwoOpening(false, true);
                    }
                    BracketFormatting::TwoOpening(_, f2) if *f2 => {
                        print_list.push(PrintInfo::new(
                            format!("{}", iner.as_span().as_str()),
                            SpaceType::ExtraIndent,
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
                            print_list.push(PrintInfo::new(format!("\n"), SpaceType::NewLine));
                            print_list.push(PrintInfo::new(
                                format!("{}", iner.as_span().as_str()),
                                SpaceType::None,
                            ));
                            print_list.push(PrintInfo::new(format!("\n"), SpaceType::NewLine));
                            *brackets = BracketFormatting::TwoOpening(false, true);
                        } else {
                            print_list.push(PrintInfo::new(
                                format!("{}", iner.as_span().as_str()),
                                SpaceType::NoRightPad,
                            ));
                        }
                    }
                    _ => {
                        print_list.push(PrintInfo::new(
                            format!("{}", iner.as_span().as_str()),
                            SpaceType::NoRightPad,
                        ));
                    }
                }
            }
            une => panic!("{:?}{}", une, iner.as_span().as_str()),
        }
    }
}

pub fn format_datatype(data_type: Pair<Rule>) -> Vec<PrintInfo> {
    let mut print_list = Vec::new();

    for iner in data_type.into_inner() {
        match iner.as_rule() {
            Rule::logical => print_list.push(PrintInfo::new(
                format!("{}", iner.as_span().as_str().to_uppercase()),
                SpaceType::None,
            )),
            Rule::array_data => print_list.append(&mut format_array(iner)),
            Rule::string => print_list.push(PrintInfo::new(format!("{}",iner.as_span().as_str()),SpaceType::None)),
            _ => print_list.push(PrintInfo::new(
                format!("{}", iner.as_span().as_str().to_uppercase()),
                SpaceType::None,
            )),
        }
    }

    print_list
}

fn format_array(array: Pair<Rule>) -> Vec<PrintInfo> {
    let mut print_list = Vec::new();
    for iner in array.into_inner() {
        match iner.as_rule() {
            Rule::left_square_bracket => print_list.push(PrintInfo::new(
                format!("{}", iner.as_span().as_str()),
                SpaceType::NoSpace,
            )),
            Rule::right_square_bracket => print_list.push(PrintInfo::new(
                format!("{}", iner.as_span().as_str()),
                SpaceType::NoLeftPad,
            )),
            Rule::comma => print_list.push(PrintInfo::new(
                format!("{}", iner.as_span().as_str()),
                SpaceType::NoLeftPad,
            )),
            Rule::variable => {
                print_list.push(PrintInfo::new(
                    format!("{}", iner.as_span().as_str()),
                    SpaceType::None,
                ));
            }
            Rule::datatype => print_list.append(&mut format_datatype(iner)),
            _ => panic!("unexpected datatype{:?}", iner.as_rule()),
        }
    }
    print_list
}

pub fn format_accumulate(accum:Pair<Rule>) -> Vec<PrintInfo> {
    let mut print_list = Vec::new();
    
    for iner in accum.into_inner(){
        match iner.as_rule(){

            Rule::accumulate_keyword | Rule::aggregate_phrase |Rule::by_keyword => {
                print_list.push(PrintInfo::new(
                    format!("{}", iner.as_span().as_str().to_uppercase()),
                    SpaceType::None,
                ));
            }
            Rule::expression => {
                print_list.append(&mut format_expression(iner, false))
            }
            Rule::COMMENT => print_list.push(PrintInfo::new(
                format!("{}", format_comment(iner)),
                SpaceType::None,
            )),
            Rule::WHITESPACE => {
                if let Some(f) = format_whitespace(iner) {
                    print_list.push(f);
                }
            }
            _ => panic!("unexpected agregate{:?}", iner.as_rule()),
            
        }
    }
    print_list
}

fn format_operator(operator: Pair<Rule>, conditional: bool) -> String {
    match operator.as_span().as_str().trim() {
        "<" => format!("LT"),
        "<=" => format!("LE"),
        ">" => format!("GT"),
        ">=" => format!("GE"),
        "<>" => format!("NE"),
        "=" if conditional => format!("EQ"),
        misc => format!("{}", misc.to_uppercase()),
    }
}
