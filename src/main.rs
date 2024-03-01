#![allow(non_snake_case)]

mod prettier;

use std::fs;
use prettier::*;

fn main() {
    let contents = fs::read_to_string("gpio_functions.s")
        .expect("[!] Could not open the file.");


    let mut comment_handler = CommentHandler::new();

    for line in contents.split('\n') {
        comment_handler.handle(line);
        if comment_handler.is_comment() {
            comment_handler.print_comment();
            continue;
        }

        let _ = is_label(line);
        let indent_size = line.find(line.trim()).unwrap();
        print!("{}", get_aligned_indent(indent_size));


        if is_instruction_format(line) {
            let comment = line.find('@');

            match comment {
                Some(comment_position) => {
                    print!("{}", prettier::normalize_command_spacing(&line[..comment_position - 1]));
                    println!("\t\t{}", &line[comment_position..]);
                }
                None => {
                    println!("{}", prettier::normalize_command_spacing(line));
                }
            };
        } else {
            println!("{}", line.trim());
        }
    }
}