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
use std::ffi::c_char;
use std::slice;
use std::str;

pub const KEYWORDS_WITH_ARGS: [&str; 70] = [
    "movs", "mov", "adds", "add", "adcs", "adr", "subs", "sbcs", "sub", "rsbs", "muls", "cmp",
    "cmn", "ands", "eors", "orrs", "bics", "mvns", "tst", "lsls", "lsrs", "asrs", "rors", "ldr",
    "ldrh", "ldrb", "ldrsh", "lsrsb", "ldm", "str", "strh", "strb", "str", "strh", "strb", "str",
    "stm", "push", "pop", "b", "bl", "bx", "blx", "beq", "bne", "bgt", "blt", "bge", "ble", "bcs",
    "bcc", "bmi", "bpl", "bvs", "bvc", "bhi", "bls", "sxth", "sxtb", "uxth", "uxtb", "rev",
    "rev16", "revsh", "svc", "cpsid", "cpsie", "mrs", "msr", "bkpt",
];

const MAX_KEYWORD_LENGTH: usize = 6;

pub struct CommentHandler {
    line: String,
    multiline: bool,
    single_line: bool,
    empty_line: bool,
}

impl CommentHandler {
    pub fn new() -> CommentHandler {
        CommentHandler {
            line: String::new(),
            multiline: false,
            single_line: false,
            empty_line: false,
        }
    }

    pub fn handle(&mut self, line: &str) {
        if line.trim().is_empty() {
            self.empty_line = true;
            return;
        }

        self.line = String::from(line);

        if self.is_single_line_comment() {
            self.single_line = true;
        } else if line.starts_with("/*") {
            self.multiline = true;
        }
    }

    pub fn get_comment(&mut self) -> String {
        if self.empty_line && !self.multiline {
            self.empty_line = false;
            return String::from("\n");
        } else if self.single_line {
            self.single_line = false;
            return self.get_single_comment();
        } else if self.line.contains("*/") {
            self.multiline = false;
            return String::from(" */\n");
        } else if self.line.contains("/*") {
            return String::from("/*\n");
        } else if self.multiline {
            return self.get_multi_comment_body();
        }

        String::new()
    }

    pub fn is_comment(&self) -> bool {
        self.single_line || self.multiline || self.empty_line
    }

    fn is_single_line_comment(&self) -> bool {
        let line_trimmed = self.line.trim();

        if line_trimmed.is_empty() || line_trimmed.starts_with('@') {
            return true;
        }
        false
    }

    fn get_multi_comment_body(&self) -> String {
        let mut comment = String::from(self.line.trim());
        if comment.starts_with('*') {
            comment.remove(0);
        }

        format!(" * {}\n", comment.trim())
    }

    fn get_single_comment(&self) -> String {
        let mut comment = self.line.clone();
        let after_comment_sign = comment.find('@').unwrap() + 1;
        let first_comment_char = comment.as_bytes()[after_comment_sign];

        if first_comment_char != (' ' as u8) && first_comment_char != ('@' as u8) {
            comment.insert(after_comment_sign, ' ');
        }

        format!("{}\n", comment)
    }
}

pub fn is_instruction_format(line: &str) -> bool {
    if is_not_instruction(line) || !starts_with_keyword(line) {
        return false;
    }

    let re = Regex::new(r"^[A-Za-z]{1,5}\s*(\w|\W)*(\s*,\s*=?#?\w+(\s*,\s*#?\w+)?)?.*$").unwrap();

    let re_stack = Regex::new(r"^[A-Za-z]{1,5}\s*\{(\w|\W|\d)*}.*$").unwrap();

    if re.is_match(line.trim()) || re_stack.is_match(line.trim()) {
        return true;
    }

    false
}

fn is_not_instruction(line: &str) -> bool {
    let line_stripped = line.trim();

    if line_stripped.is_empty()
        || line_stripped.starts_with('.')
        || line_stripped.starts_with('@')
        || line_stripped.starts_with("/*")
        || line_stripped.contains("*/")
        || line_stripped.starts_with("*")
    {
        return true;
    }

    false
}

fn starts_with_keyword(line: &str) -> bool {
    let first_token = line.split_whitespace().collect::<Vec<&str>>()[0].to_ascii_lowercase();

    KEYWORDS_WITH_ARGS.contains(&first_token.as_str())
}

fn get_keyword_spaces(keyword: &str) -> String {
    if !KEYWORDS_WITH_ARGS.contains(&keyword) {
        panic!("[!] `{}` is not a valid keyword. Aborting.", keyword);
    }

    let spaces_amount = MAX_KEYWORD_LENGTH - keyword.len();

    String::from_utf8(vec![b' '; spaces_amount]).unwrap()
}

pub fn normalize_command_spacing(command: &str) -> String {
    let tokens: Vec<&str> = command
        .split_terminator(&[' ', '\t', '\r', ','])
        .filter(|&x| !x.is_empty())
        .collect();

    let mut normalized_command = String::from(tokens[0]);

    if tokens.len() == 1 {
        return normalized_command;
    }
    normalized_command += get_keyword_spaces(&normalized_command).as_str();

    normalized_command += &tokens[1..].join(", ");
    normalized_command
}

pub fn get_aligned_indent(indent: usize) -> String {
    let spaces_amount = indent.div_ceil(4) * 4;

    String::from_utf8(vec![b' '; spaces_amount]).unwrap()
}

pub fn is_label(line: &str) -> bool {
    if line.split_whitespace().collect::<Vec<_>>().len() == 1 && line.trim().ends_with(':') {
        return true;
    }

    false
}

pub fn format(contents: &str) -> String {
    let mut comment_handler = CommentHandler::new();
    let mut output = String::new();

    for line in contents.split('\n') {
        comment_handler.handle(line);
        if comment_handler.is_comment() {
            output += comment_handler.get_comment().as_str();
            continue;
        }

        let _ = is_label(line);
        let indent_size = line.find(line.trim()).unwrap();
        output += get_aligned_indent(indent_size).as_str();

        if is_instruction_format(line) {
            let comment = line.find('@');

            match comment {
                Some(comment_position) => {
                    output += normalize_command_spacing(&line[..comment_position - 1]).as_str();
                    output += format!("\t\t\t{}\n", &line[comment_position..]).as_str();
                }
                None => {
                    output += format!("{}\n", normalize_command_spacing(line)).as_str();
                }
            };
        } else {
            output += format!("{}\n", line.trim()).as_str();
        }
    }

    output
}

#[no_mangle]
pub unsafe extern "C" fn format_arm_asm_code(
    contents: *const c_char,
    length: usize,
) -> *mut c_char {
    if contents.is_null() || std::ptr::read(contents) as u8 == 0u8 {
        return std::ptr::null_mut();
    }

    let contents_str =
        str::from_utf8_unchecked(slice::from_raw_parts(contents as *const u8, length));

    let formatted_contents = format(contents_str);
    let formatted_contents_length = formatted_contents.as_bytes().len();
    let null_terminated_length = formatted_contents_length + 1;

    let formatted_str: *mut c_char = libc::malloc(null_terminated_length).cast();
    std::ptr::copy(
        formatted_contents.as_str().as_bytes().as_ptr().cast(),
        formatted_str,
        formatted_contents_length,
    );
    std::ptr::write(
        formatted_str.offset(formatted_contents_length as isize) as *mut u8,
        0u8,
    );

    formatted_str
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
    fn test_normalize_command_spacing() {
        assert_eq!(normalize_command_spacing("ldr r1, r2"), "ldr   r1, r2");
        assert_eq!(normalize_command_spacing("b        .init"), "b     .init");
        assert_eq!(
            normalize_command_spacing("adds r0 , r1, r2"),
            "adds  r0, r1, r2"
        );
        assert_eq!(
            normalize_command_spacing("adds r0 , r1 , r2"),
            "adds  r0, r1, r2"
        );
    }
}
