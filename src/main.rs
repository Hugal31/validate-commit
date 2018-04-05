extern crate validate_commit;

use std::process::exit;

fn main() {
    if std::env::args().len() != 2 {
        eprintln!("Need one argument");
        exit(1);
    }

    let file_path = std::env::args()
        .nth(1)
        .unwrap();
    if let Err(e) = validate_commit::validate_commit_file(&file_path) {
        eprintln!("error: {}", e);
        exit(1);
    }
}
