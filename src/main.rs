use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use std::env;
use std::io;
use std::process;

const CHARACTER_CLASS: u8 = b'\\';
const CHARACTER_ALPHA: u8 = b'w';
const CHARACTER_DIGIT: u8 = b'd';

fn match_match_group(input_line: &str, pattern: &str) -> Result<bool> {
    let is_negative = pattern.chars().next().expect("no group speciefied") == '^';
    let skip_chars = if is_negative { 1 } else { 0 };
    let group = pattern.chars().skip(skip_chars);
    Ok(group
        .into_iter()
        .any(|c| input_line.contains(c) != is_negative))
}

fn match_characters(input_line: &str, pattern: &str) -> Result<bool> {
    for (i, _) in input_line.char_indices() {
        if match_characters_exact(&input_line[i..], pattern)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn match_characters_exact(input_line: &str, pattern: &str) -> Result<bool> {
    let mut found_character_class = false;
    let mut input_index = 0;
    let input = input_line.as_bytes();
    let pattern = pattern.as_bytes();

    for c in pattern {
        let found = c == &CHARACTER_CLASS;

        if !found {
            if input_index >= input_line.len() {
                return Ok(false);
            }

            let is_ok = if found_character_class {
                match_character_class(input[input_index] as char, *c)?
            } else {
                match_character(input[input_index], *c)?
            };

            if !is_ok {
                return Ok(false);
            }

            input_index += 1;
        }

        found_character_class = found;
    }
    Ok(true)
}

fn match_character(input_line: u8, pattern: u8) -> Result<bool> {
    Ok(input_line == pattern)
}

fn match_character_class(input_line: char, pattern: u8) -> Result<bool> {
    match pattern {
        CHARACTER_ALPHA => Ok(input_line.is_ascii_alphanumeric()),
        CHARACTER_DIGIT => Ok(input_line.is_ascii_digit()),
        _ => bail!("Unhandled pattern: {}", pattern),
    }
}

fn match_pattern(input_line: &str, pattern: &str) -> Result<bool> {
    if pattern.starts_with('[') && pattern.ends_with(']') {
        let count = pattern.len();
        match_match_group(input_line, &pattern[1..count - 2])
    } else {
        match_characters(input_line, pattern)
    }
}

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() -> Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    if env::args().nth(1).context("no input found")? != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }

    let pattern = env::args().nth(2).context("no pattern found")?;
    let mut input_line = String::new();

    io::stdin().read_line(&mut input_line)?;

    if match_pattern(&input_line, &pattern)? {
        println!("pattern was matching");
        process::exit(0)
    } else {
        println!("pattern is not matching");
        process::exit(1)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Error;

    use super::*;

    fn match_result(result: Result<bool, Error>, expected: bool) {
        match result {
            Ok(r) => assert_eq!(r, expected),
            Err(_) => panic!(""),
        }
    }

    #[test]
    fn match_single_character() {
        let result = match_pattern("apple", "a");
        match_result(result, true);
    }

    #[test]
    fn match_digit() {
        let result = match_pattern("apple123", "\\d");
        match_result(result, true);
    }

    #[test]
    fn match_no_digit() {
        let result = match_pattern("apple", "\\d");
        match_result(result, false);
    }

    #[test]
    fn match_alphanumeric() {
        let result = match_pattern("apple", "\\w");
        match_result(result, true);
    }

    #[test]
    fn match_no_alphanumeric() {
        let result = match_pattern("$!?", "\\w");
        match_result(result, false);
    }

    #[test]
    fn match_positive_match_group() {
        let result = match_pattern("apple", "[abc]");
        match_result(result, true);
    }

    #[test]
    fn match_no_positive_match_group() {
        let result = match_pattern("apple", "[bcd]");
        match_result(result, false);
    }

    #[test]
    fn match_negative_match_group() {
        let result = match_pattern("apple", "[^abc]");
        match_result(result, true);
    }

    #[test]
    fn match_no_negative_match_group() {
        let result = match_pattern("cab", "[^abc]");
        match_result(result, false);
    }

    #[test]
    fn match_combined_character_classes() {
        let result = match_pattern("3 dogs", "\\d \\w\\w\\ws");
        match_result(result, true);
    }

    #[test]
    fn match_no_combined_character_classes() {
        let result = match_pattern("sally has 124 apples", "\\d\\d\\d apples");
        match_result(result, true);
    }
}
