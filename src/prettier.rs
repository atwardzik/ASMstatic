/*
 *  Copyright (C) 2024  Artur Twardzik
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use regex::Regex;

pub const KEYWORDS_WITH_ARGS: [&str; 70] = [
    "movs", "mov", "adds", "add", "adcs", "adr", "subs", "sbcs", "sub", "rsbs", "muls",
    "cmp", "cmn", "ands", "eors", "orrs", "bics", "mvns", "tst", "lsls", "lsrs", "asrs",
    "rors", "ldr", "ldrh", "ldrb", "ldrsh", "lsrsb", "ldm", "str", "strh", "strb", "str",
    "strh", "strb", "str", "stm", "push", "pop", "b", "bl", "bx", "blx", "beq", "bne",
    "bgt", "blt", "bge", "ble", "bcs", "bcc", "bmi", "bpl", "bvs", "bvc", "bhi", "bls",
    "sxth", "sxtb", "uxth", "uxtb", "rev", "rev16", "revsh", "svc", "cpsid", "cpsie",
    "mrs", "msr", "bkpt"
];

const MAX_KEYWORD_LENGTH: usize = 6;


pub fn is_instruction_format(line: &str) -> bool {
    if is_not_instruction(line) || !starts_with_keyword(line) {
        return false;
    }

    let re = Regex::new(
        r"^[A-Za-z]{1,5}\s*(\w|\W)*(\s*,\s*=?#?\w+(\s*,\s*#?\w+)?)?.*$")
        .unwrap();

    let re_stack = Regex::new(
        r"^[A-Za-z]{1,5}\s*\{(\w|\W|\d)*}.*$")
        .unwrap();

    if re.is_match(line.trim()) || re_stack.is_match(line.trim()) {
        return true;
    }

    false
}

fn is_not_instruction(line: &str) -> bool {
    let line_stripped = line.trim();

    if line_stripped.is_empty() || line_stripped.starts_with('.') ||
        line_stripped.starts_with('@') || line_stripped.starts_with("/*") ||
        line_stripped.contains("*/") || line_stripped.starts_with("*") {
        return true;
    }

    false
}

fn starts_with_keyword(line: &str) -> bool {
    let first_token = line.split_whitespace()
        .collect::<Vec<&str>>()[0]
        .to_ascii_lowercase();

    let is_token = KEYWORDS_WITH_ARGS.iter()
        .any(|&token| token == &first_token);

    return is_token;
}

pub fn normalize_keyword_spacing(line: &str) -> String {
    let tokens: Vec<&str> = line.split_whitespace().collect();

    let keyword_position = line.find(tokens[0]).unwrap();
    let keyword_length = tokens[0].len();
    let argument_position = line.find(tokens[1]).unwrap();

    let spaces_amount = MAX_KEYWORD_LENGTH - keyword_length;

    let mut transformed_line: String = String::new();
    transformed_line += &line[..keyword_position];
    transformed_line += tokens[0];
    transformed_line += String::from_utf8(vec![b' '; spaces_amount]).unwrap().as_str();
    transformed_line += &line[argument_position..];

    transformed_line
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_instruction_format() {
        assert!(is_instruction_format("cpsid i"));
        assert!(is_instruction_format("mov r0, r1"));
        assert!(is_instruction_format("mov r0, #5"));
        assert!(is_instruction_format("adds r0, r1, r2"));
        assert!(is_instruction_format("adds r0 , r1, #0x12"));
        assert!(is_instruction_format("push {r0-r7, lr}"));
        assert!(is_instruction_format("ldr r0, variable"));
        assert!(is_instruction_format("ldr r0, variable"));
        assert!(is_instruction_format("ldr r0, variable \t @ with comment"));
        assert!(is_instruction_format("b .exit"));

        assert!(!is_instruction_format("@ rgeq"));
    }

    #[test]
    fn test_is_not_instruction() {
        assert!(is_not_instruction("@ this is a comment with subs and mov"));
        assert!(is_not_instruction("/* adds"));
        assert!(is_not_instruction(" eors */"));
        assert!(is_not_instruction(" * comment body with push"));
        assert!(is_not_instruction(".thumb_func"));
        assert!(is_not_instruction("\n"));

        assert!(!is_not_instruction(" mov r0, r1"));
    }

    #[test]
    fn test_starts_with_keyword() {
        assert!(starts_with_keyword(" mOV r0,  r1 "));

        assert!(!starts_with_keyword("  @ ireugbewufbqi"));
    }

    #[test]
    fn test_normalize_keyword_spacing() {
        assert_eq!(normalize_keyword_spacing("ldr r1, r2"), "ldr   r1, r2");
        assert_eq!(normalize_keyword_spacing("b        .init"), "b     .init");
    }
}
