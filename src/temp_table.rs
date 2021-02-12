use crate::{format_comment, format_whitespace, PrintInfo, Rule, SpaceType};
use pad::PadStr;
use pest::iterators::{Pair, Pairs};

struct FieldInfo {
    before_variable: String,
    variable: String,
    variable_length: usize,
    after_variable: String,
}

impl FieldInfo {
    fn new(
        before_variable: String,
        vairable: String,
        variable_length: usize,
        after_variable: String,
    ) -> Self {
        Self {
            before_variable,
            variable: vairable,
            variable_length,
            after_variable,
        }
    }
}

pub fn format_temp_table(temp_table: Pair<Rule>) -> Vec<PrintInfo> {
    let mut print_list = Vec::new();

    for iner in temp_table.into_inner() {
        match iner.as_rule() {
            Rule::define_keyword | Rule::keyword | Rule::temptable_keyword => {
                print_list.push(PrintInfo::new(
                    format!("{}", iner.as_span().as_str().to_uppercase()),
                    SpaceType::None,
                ))
            }
            Rule::variable => print_list.push(PrintInfo::new(
                format!("{}", iner.as_span().as_str()),
                SpaceType::None,
            )),
            Rule::temp_table_like => print_list.append(&mut format_temp_table_like(iner)),
            Rule::temp_table_fields => print_list.append(&mut format_temp_table_fields(iner)),
            Rule::WHITESPACE | Rule::NEWLINE => {
                if let Some(f) = format_whitespace(iner) {
                    print_list.push(f);
                }
            }
            _ => eprintln!("@@@temp-table-not handled"),
        }
    }
    print_list
}

fn format_temp_table_like(like_field: Pair<Rule>) -> Vec<PrintInfo> {
    let mut print_list = Vec::new();

    for iner in like_field.into_inner() {
        match iner.as_rule() {
            Rule::like_keyword | Rule::keyword => print_list.push(PrintInfo::new(
                format!("{}", iner.as_span().as_str().to_uppercase()),
                SpaceType::None,
            )),
            Rule::variable => print_list.push(PrintInfo::new(
                format!("{}", iner.as_span().as_str()),
                SpaceType::None,
            )),
            Rule::WHITESPACE | Rule::NEWLINE => {
                if let Some(f) = format_whitespace(iner) {
                    print_list.push(f);
                }
            }
            _ => panic!("temp table like not formatted"),
        }
    }
    print_list
}

fn format_temp_table_fields(fields: Pair<Rule>) -> Vec<PrintInfo> {
    let mut fields_and_length_list = Vec::new();
    let mut largest_variable_length = 0;
    for iner in fields.into_inner() {
        let next_field = format_temp_table_single_field(iner);
        if next_field.variable_length > largest_variable_length {
            largest_variable_length = next_field.variable_length;
        }
        fields_and_length_list.push(next_field);
    }
    let print_list_alloc_len = fields_and_length_list.len();
    let print_list = fields_and_length_list.into_iter().fold(
        Vec::with_capacity(print_list_alloc_len),
        |mut acc, mut curr| -> Vec<PrintInfo> {
            let mut line = String::new();
            /*line.push_str(&format!(" "));*/
            line.push_str(&mut curr.before_variable);
            line.push_str(&format!(" "));
            line.push_str(&mut curr.variable.pad_to_width(largest_variable_length));
            // line.push_str(&format!(" "));
            line.push_str(&mut curr.after_variable);

            acc.push(PrintInfo::new(line, SpaceType::None));
            acc.push(PrintInfo::new(format!("\n"), SpaceType::NewLine));
            acc
        },
    );
    print_list
}

fn format_temp_table_single_field(single_field: Pair<Rule>) -> FieldInfo {
    //println!("$$$Single FIELD{:?}$$$$$", single_field);
    let mut field_iterator = single_field.into_inner();
    loop {
        /*let next = match field_iterator.next() {
            Some(x) => x,
            None => break,
        };*/
        let (before_variable, variable) =
            format_temp_table_single_field_process_to_variable(&mut field_iterator);
        let after_variable = format_temp_table_single_filed_to_end(&mut field_iterator);
        //println!("####{}###", after_variable);
        let variable_len = variable.len();
        break FieldInfo::new(before_variable, variable, variable_len, after_variable);
    }
}

fn format_temp_table_single_field_process_to_variable(
    field_iterator: &mut Pairs<Rule>,
) -> (String, String) {
    let mut before_variable_string = String::new();
    let variable = loop {
        let next = match field_iterator.next() {
            Some(x) => x,
            None => panic!("invalid pass through temp-table"),
        };

        match next.as_rule() {
            Rule::field_keyword | Rule::aslike_keyword | Rule::keyword => {
                before_variable_string.push_str(&next.as_str().to_uppercase())
            }
            Rule::variable => break next.as_str().to_string(),
            Rule::WHITESPACE | Rule::NEWLINE => {
                if let Some(f) = format_whitespace(next) {
                    before_variable_string.push_str(&f.line);
                }
            }
            Rule::COMMENT => before_variable_string.push_str(&format_comment(next)),
            _ => panic!("unexpected format to variable"),
        }
    };
    (before_variable_string, variable)
}

fn format_temp_table_single_filed_to_end(field_iterator: &mut Pairs<Rule>) -> String {
    field_iterator.fold(String::new(), |mut accum, curr| {
        match curr.as_rule() {
            Rule::keyword => accum.push_str(&curr.as_str().to_uppercase()),
            _ => {
                accum.push_str(curr.as_str());
            }
        }
        accum
    })
}
