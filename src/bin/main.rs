extern crate clap;
use abl_formatter::*;
use clap::{App, Arg};
fn main() {
    let options = App::new("Recut")
    .version("0.1")
    .about("Similar to cut but with unicode support and inferred delimiters")
    .arg(
        Arg::with_name("FILE")
        .index(1)
        .help("The file and (Accompanying path if neccessary ) to process standard input if empty or -- Standard input is used")
    )   .get_matches();
    let input_type = match options.value_of("FILE") {
        Some(s) if s != "-" && s != "--" => IoType::FromFile(s.to_owned()),
        _ => IoType::FromStdIn,
    };

    format_code(input_type);
}
