#![allow(non_snake_case)]

mod prettier;

use prettier::*;
use std::fs;

fn main() {
    let contents = fs::read_to_string("gpio_functions.s").expect("[!] Could not open the file.");

    println!("{}", format(contents.as_str()));
}
