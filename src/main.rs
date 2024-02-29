#![allow(non_snake_case)]

mod prettier;

use std::fs;

fn main() {
    let contents = fs::read_to_string("gpio_functions.s")
        .expect("[!] Could not open the file.");

    for line in contents.split('\n') {
        if prettier::is_instruction_format(line) {
            println!("{}", prettier::normalize_keyword_spacing(line));
        } else {
            println!("{}", line);
        }
    }
}