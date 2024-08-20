use anyhow::bail;
use anyhow::Result;

mod test;

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

pub fn match_pattern(input_line: &str, pattern: &str) -> Result<bool> {
    if pattern.starts_with('[') && pattern.ends_with(']') {
        let count = pattern.len();
        match_match_group(input_line, &pattern[1..count - 2])
    } else {
        match_characters(input_line, pattern)
    }
}
