use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use std::env;
use std::io;
use std::process;

fn match_pattern(input_line: &str, pattern: &str) -> Result<bool> {
    match pattern.chars().count() {
        1 => Ok(input_line.contains(pattern)),
        2 => {
            if pattern.starts_with('\\') {
                match pattern.chars().nth(1).expect("invalid pattern") {
                    'd' => Ok(input_line.chars().any(|c| c.is_ascii_digit())),
                    'w' => Ok(input_line.chars().any(|c| c.is_ascii_alphanumeric())),

                    _ => bail!("Unhandled pattern: {}", pattern),
                }
            } else {
                bail!("Unhandled pattern: {}", pattern)
            }
        }
        count => {
            if pattern.starts_with('[') && pattern.ends_with(']') {
                let is_negative = pattern.chars().nth(1).expect("no group speciefied") == '^';
                let skip_chars = if is_negative { 2 } else { 1 };
                let group = pattern.chars().skip(skip_chars).take(count - 2);
                Ok(if is_negative {
                    group.into_iter().all(|c| !input_line.contains(c))
                } else {
                    group.into_iter().any(|c| input_line.contains(c))
                })
            } else {
                bail!("Unhandled pattern: {}", pattern)
            }
        }
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
