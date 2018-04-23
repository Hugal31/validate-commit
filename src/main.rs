extern crate termcolor;
extern crate validate_commit;

use std::io::Write;
use std::process::exit;

use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

fn main() {
    if std::env::args().len() != 2 {
        eprintln!("Need one argument");
        exit(1);
    }

    let file_path = std::env::args().nth(1).unwrap();
    if let Err(e) = validate_commit::validate_commit_file(&file_path) {
        write_error(&e);
        exit(1);
    }
}

fn write_error(error: &validate_commit::CommitValidationError) {
    let formatted_error = format!("{}", error);
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    stdout
        .set_color(ColorSpec::new().set_bold(true).set_fg(Some(Color::Red)))
        .and_then(|()| stdout.write_all(b"error: "))
        .and_then(|()| stdout.reset())
        .and_then(|()| stdout.write_fmt(format_args!("{}\n", formatted_error)))
        .expect(&formatted_error);
}
