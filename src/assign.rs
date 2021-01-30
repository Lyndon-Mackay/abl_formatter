use std::env::var;

use crate::{format_expression, format_whitespace, PrintInfo, PrintType, Rule};
use pad::PadStr;
use pest::iterators::{Pair, Pairs};

struct AssignLinePrintInfo {
    before_var: Vec<PrintInfo>,
    variable: String,
    variable_len: usize,
    before_assign_expr: Vec<PrintInfo>,
    assign_expr: Vec<PrintInfo>,
    pre_when: Vec<PrintInfo>,
    when: Option<PrintInfo>,
    pre_when_expr: Vec<PrintInfo>,
    when_expr: Vec<PrintInfo>,
    end: Vec<PrintInfo>,
    estimated_total: usize,
}

impl AssignLinePrintInfo {
    fn new(
        before_var: Vec<PrintInfo>,
        variable: String,
        before_assign_expr: Vec<PrintInfo>,
        assign_expr: Vec<PrintInfo>,
        pre_when: Vec<PrintInfo>,
        when: Option<PrintInfo>,
        pre_when_expr: Vec<PrintInfo>,
        when_expr: Vec<PrintInfo>,
        end: Vec<PrintInfo>,
    ) -> Self {
        let estimated_total = before_var.len()
            + variable.len()
            + before_assign_expr.len()
            + before_assign_expr.len()
            + assign_expr.len()
            + pre_when.len()
            + match when {
                Some(ref x) => x.line.len(),
                None => 0,
            }
            + pre_when_expr.len()
            + when_expr.len()
            + end.len();
        let variable_len = variable.len();

        Self {
            before_var,
            variable,
            variable_len,
            before_assign_expr,
            assign_expr,
            pre_when,
            when,
            pre_when_expr,
            when_expr,
            end,
            estimated_total,
        }
    }
}

pub fn format_assign(assign: Pair<Rule>) -> Vec<PrintInfo> {
    let mut print_list = Vec::new();

    for iner in assign.into_inner() {
        match iner.as_rule() {
            Rule::assign_keyword
            | Rule::input_keyword
            | Rule::framebrowse_keyword
            | Rule::noerror_keyword => print_list.push(PrintInfo::new(
                format!("{}", iner.as_span().as_str().to_uppercase()),
                PrintType::None,
            )),
            Rule::variable => print_list.push(PrintInfo::new(
                format!("{}", iner.as_span().as_str()),
                PrintType::None,
            )),
            Rule::assign_lines => print_list.append(&mut format_assign_lines(iner)),
            Rule::WHITESPACE => {}

            une => panic!(" unexpected assign {:?}", une),
        }
    }
    print_list
}

fn format_assign_lines(lines: Pair<Rule>) -> Vec<PrintInfo> {
    let (longest, lines_breakdown) =
        lines
            .into_inner()
            .fold((0, Vec::new()), |(longest, mut accum), curr| {
                let curr_line = format_assign_single_line(curr);
                let len = curr_line.variable_len;
                accum.push(curr_line);
                if len > longest {
                    (len, accum)
                } else {
                    (longest, accum)
                }
            });
    lines_breakdown
        .into_iter()
        .flat_map(|mut x| {
            let mut inner_list = Vec::with_capacity(x.estimated_total);

            inner_list.append(&mut x.before_var);
            inner_list.push(PrintInfo::new(
                x.variable.pad_to_width(longest),
                PrintType::None,
            ));
            inner_list.append(&mut x.before_assign_expr);
            inner_list.append(&mut x.assign_expr);
            inner_list.append(&mut x.pre_when);
            if let Some(when) = x.when {
                inner_list.push(when);
            }
            inner_list.append(&mut x.pre_when_expr);
            inner_list.append(&mut x.when_expr);
            inner_list.append(&mut x.end);

            inner_list
        })
        .collect()
}

fn format_assign_single_line(line: Pair<Rule>) -> AssignLinePrintInfo {
    let mut iter = line.into_inner();

    let (before_var, variable) = get_to_variable(&mut iter);
    let (before_assign_expr, assign_expr) = get_to_assign_expression(&mut iter);
    let (pre_when, when_keyword) = get_to_when_keyword(&mut iter);

    let (pre_when_expr, when_expr, post_expr) = match when_keyword {
        Some(_) => {
            let (before_when_expr, when_expr) = get_to_expression(&mut iter);
            let post_expr = iter
                .filter_map(|x| {
                    if let Rule::COMMENT = x.as_rule() {
                        Some(PrintInfo::new(x.as_str().to_string(), PrintType::None))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            (before_when_expr, when_expr, post_expr)
        }
        None => (Vec::new(), Vec::new(), Vec::new()),
    };

    AssignLinePrintInfo::new(
        before_var,
        variable,
        before_assign_expr,
        assign_expr,
        pre_when,
        when_keyword,
        pre_when_expr,
        when_expr,
        post_expr,
    )
}

fn get_to_variable(iter: &mut Pairs<Rule>) -> (Vec<PrintInfo>, String) {
    let mut before_var = Vec::new();

    let variable = loop {
        let next = iter.next().expect("invalid assign");
        match next.as_rule() {
            Rule::COMMENT => {
                before_var.push(PrintInfo::new(next.as_str().to_string(), PrintType::None))
            }
            Rule::variable => break next.as_str().to_string(),
            Rule::WHITESPACE => {}
            une => panic!(" unexpected assign {:?}", une),
        }
    };
    (before_var, variable)
}

fn get_to_assign_expression(iter: &mut Pairs<Rule>) -> (Vec<PrintInfo>, Vec<PrintInfo>) {
    let mut before_expr = Vec::new();
    let expr = loop {
        let next = iter.next().expect("invalid assign");
        match next.as_rule() {
            Rule::COMMENT => {
                before_expr.push(PrintInfo::new(next.as_str().to_string(), PrintType::None))
            }
            Rule::expression => break format_expression(next, true),
            Rule::equals => {
                before_expr.push(PrintInfo::new(next.as_str().to_string(), PrintType::None))
            }
            Rule::WHITESPACE => {}
            une => panic!(" unexpected assign {:?}", une),
        }
    };
    (before_expr, expr)
}

fn get_to_when_keyword(iter: &mut Pairs<Rule>) -> (Vec<PrintInfo>, Option<PrintInfo>) {
    let mut before_when = Vec::new();
    let when = loop {
        let next = match iter.next() {
            Some(item) => item,
            None => break None,
        };
        match next.as_rule() {
            Rule::COMMENT => {
                before_when.push(PrintInfo::new(next.as_str().to_string(), PrintType::None))
            }
            Rule::when_keyword => {
                break Some(PrintInfo::new(next.as_str().to_string(), PrintType::None))
            }
            Rule::WHITESPACE => {}
            une => panic!(" unexpected assign {:?}", une),
        }
    };
    (before_when, when)
}

fn get_to_expression(iter: &mut Pairs<Rule>) -> (Vec<PrintInfo>, Vec<PrintInfo>) {
    let mut before_expr = Vec::new();
    let expr = loop {
        let next = iter.next().expect("when with no expression");

        match next.as_rule() {
            Rule::COMMENT => {
                before_expr.push(PrintInfo::new(next.as_str().to_string(), PrintType::None))
            }
            Rule::expression => break format_expression(next, true),
            Rule::WHITESPACE => {}
            une => panic!(" unexpected assign {:?}", une),
        }
    };
    (before_expr, expr)
}
