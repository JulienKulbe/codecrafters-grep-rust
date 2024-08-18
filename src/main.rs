use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use std::env;
use std::io;
use std::process;

const CHARACTER_CLASS: u8 = b'\\';
const CHARACTER_ALPHA: u8 = b'w';
const CHARACTER_DIGIT: u8 = b'd';
const START_ANCHOR: u8 = b'^';
const END_ANCHOR: u8 = b'$';
const ONE_OR_MORE: u8 = b'+';
const ZERO_OR_ONE: u8 = b'?';

#[derive(Copy, Clone)]
enum MatchingType {
    /// Simple types are matching exactly one time (no postfix operator)
    Simple(CharacterType),
    /// Multiple types (+) are matching one or more times
    Multiple(CharacterType),
    /// Optional types (?) are matching zero or one time
    Optional(CharacterType),
}

#[derive(Copy, Clone)]
enum CharacterType {
    /// Character type is a character that matches exactly that character, e.g. 'a'
    Character(u8),
    /// Class types are a set of characters that can match the input
    Class(CharacterClass),
}

#[derive(Copy, Clone)]
enum CharacterClass {
    /// Character class that matches all ascii alpha numeric inputs
    Alpha,
    /// Character class that matches only digits
    Digit,
}

enum MatchResult {
    Positive(PositiveMatchResult),
    Negative,
}

struct PositiveMatchResult {
    pattern_chars: usize,
    input_chars: usize,
}

impl MatchingType {
    fn get_type(pattern: &[u8]) -> Result<MatchingType> {
        let character = CharacterType::get_type(pattern)?;

        if pattern.len() > character.len() {
            match pattern[character.len()] {
                ONE_OR_MORE => Ok(MatchingType::Multiple(character)),
                ZERO_OR_ONE => Ok(MatchingType::Optional(character)),
                _ => Ok(MatchingType::Simple(character)),
            }
        } else {
            Ok(MatchingType::Simple(character))
        }
    }

    fn matches(&self, input: &[u8]) -> MatchResult {
        match self {
            MatchingType::Simple(c) => {
                if input.is_empty() {
                    MatchResult::Negative
                } else {
                    c.matches(input[0])
                }
            }
            MatchingType::Multiple(c) => {
                let matches = c.match_count(input);
                MatchResult::new(matches > 0, c.len() + 1, matches)
            }
            MatchingType::Optional(c) => {
                let matches = c.match_count(input);
                MatchResult::new(matches < 2, c.len() + 1, matches)
            }
        }
    }
}

impl CharacterType {
    fn get_type(pattern: &[u8]) -> Result<CharacterType> {
        match pattern[0] {
            CHARACTER_CLASS => CharacterClass::get_type(pattern[1]),
            _ => Ok(CharacterType::Character(pattern[0])),
        }
    }

    fn matches(&self, input: u8) -> MatchResult {
        match self {
            CharacterType::Character(c) => MatchResult::new(&input == c, 1, 1),
            CharacterType::Class(class) => class.matches(input),
        }
    }

    fn len(&self) -> usize {
        match self {
            CharacterType::Character(_) => 1,
            CharacterType::Class(_) => 2,
        }
    }

    fn match_count(&self, input: &[u8]) -> usize {
        input
            .iter()
            .take_while(|i| self.matches(**i).is_matching())
            .count()
    }
}

impl CharacterClass {
    fn get_type(pattern: u8) -> Result<CharacterType> {
        match pattern {
            CHARACTER_ALPHA => Ok(CharacterType::Class(CharacterClass::Alpha)),
            CHARACTER_DIGIT => Ok(CharacterType::Class(CharacterClass::Digit)),
            _ => bail!("Unhandled pattern: \\{}", pattern),
        }
    }

    fn matches(&self, input: u8) -> MatchResult {
        let result = match self {
            CharacterClass::Alpha => input.is_ascii_alphanumeric(),
            CharacterClass::Digit => input.is_ascii_digit(),
        };
        MatchResult::new(result, 2, 1)
    }
}

impl MatchResult {
    fn new(result: bool, pattern_chars: usize, input_chars: usize) -> MatchResult {
        if result {
            MatchResult::ok(pattern_chars, input_chars)
        } else {
            MatchResult::Negative
        }
    }

    fn ok(pattern_chars: usize, input_chars: usize) -> MatchResult {
        MatchResult::Positive(PositiveMatchResult {
            pattern_chars,
            input_chars,
        })
    }

    fn is_matching(&self) -> bool {
        match self {
            MatchResult::Positive(_) => true,
            MatchResult::Negative => false,
        }
    }
}

fn match_match_group(input_line: &str, pattern: &str) -> Result<bool> {
    let is_negative = pattern.chars().next().expect("no group speciefied") == '^';
    let skip_chars = if is_negative { 1 } else { 0 };
    let group = pattern.chars().skip(skip_chars);
    Ok(group
        .into_iter()
        .any(|c| input_line.contains(c) != is_negative))
}

fn match_characters(input_line: &str, pattern: &str) -> Result<bool> {
    let mut pattern_len = pattern.len();
    let has_start_anchor = pattern.as_bytes()[0] == START_ANCHOR;
    let has_end_anchor = pattern.as_bytes()[pattern_len - 1] == END_ANCHOR;

    if has_start_anchor {
        pattern_len -= 1;
    }
    if has_end_anchor {
        pattern_len -= 1;
    }

    if has_start_anchor {
        match_characters_exact(input_line, &pattern[1..])
    } else if has_end_anchor {
        match_characters_exact(
            &input_line[(input_line.len() - pattern_len)..],
            &pattern[..pattern.len() - 1],
        )
    } else {
        match_characters_iterate(input_line, pattern)
    }
}

fn match_characters_iterate(input_line: &str, pattern: &str) -> Result<bool> {
    for (i, _) in input_line.char_indices() {
        if match_characters_exact(&input_line[i..], pattern)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn match_characters_exact(input_line: &str, pattern: &str) -> Result<bool> {
    let mut input_index = 0;
    let mut pattern_index = 0;
    let input = input_line.as_bytes();
    let pattern = pattern.as_bytes();

    while pattern_index < pattern.len() {
        let current_pattern = &pattern[pattern_index..];
        let current_input = &input[input_index..];

        let char_type = MatchingType::get_type(current_pattern)?;
        let result = char_type.matches(current_input);

        match result {
            MatchResult::Positive(result) => {
                pattern_index += result.pattern_chars;
                input_index += result.input_chars;
            }
            MatchResult::Negative => return Ok(false),
        }
    }

    Ok(true)
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
    fn match_start_anchor() {
        let result = match_pattern("log", "^log");
        match_result(result, true);
    }

    #[test]
    fn match_no_start_anchor() {
        let result = match_pattern("slog", "^log");
        match_result(result, false);
    }

    #[test]
    fn match_end_anchor() {
        let result = match_pattern("dog", "dog$");
        match_result(result, true);
    }

    #[test]
    fn match_no_end_anchor() {
        let result = match_pattern("dogs", "dog$");
        match_result(result, false);
    }

    #[test]
    fn match_one_or_more_times() {
        let result = match_pattern("SaaS", "a+");
        match_result(result, true);
    }

    #[test]
    fn match_no_one_or_more_times() {
        let result = match_pattern("dog", "a+");
        match_result(result, false);
    }

    #[test]
    fn match_zero_or_one_time() {
        let result = match_pattern("dog", "dogs?");
        match_result(result, true);
    }
}
