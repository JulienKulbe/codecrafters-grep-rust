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

                    _ => bail!("Unhandled pattern: {}", pattern),
                }
            } else {
                bail!("Unhandled pattern: {}", pattern)
            }
        }
        _ => bail!("Unhandled pattern: {}", pattern),
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
