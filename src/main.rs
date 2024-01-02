use clap::Parser;
use std::{
    fs::File,
    io::{self, BufReader, Read},
};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // Optional file to run operations on
    file_name: Option<String>,
}

// Use a stack for brackets
// Use a boolean for when we're inside strings, while handling escaped quotes (return false if we encounter escaping outside of string)
// Refer to json.org for parsing logic

// Can largely ignore white space unless we're in a string// Use a stack for brackets
// Use a boolean for when we're inside strings, while handling escaped quotes
// Refer to json.org for parsing logic

// Can largely ignore white space unless we're in a string
fn main() -> io::Result<()> {
    let args = Args::parse();

    let json = get_json(args.file_name)?;
    println!("{:?}", json.chars());
    let is_valid = parse_json(&json);

    if is_valid {
        println!("{}", 0);
    } else {
        println!("{}", 1)
    }
    Ok(())
}

fn parse_json(json: &String) -> bool {
    if json.is_empty() {
        return false;
    }

    let mut bracket_stack: Vec<char> = Vec::new();
    let mut is_in_string = false;
    let mut prev_char: char = '{';
    let chars = json.chars();

    for char in chars {
        if !is_in_string {
            if char == '"' {
                is_in_string = true;
            } else if char == '{' {
                bracket_stack.push(char);
            } else if char == '}' {
                // Verify bracket closures
                if bracket_stack[bracket_stack.len() - 1] == '{' {
                    bracket_stack.pop();
                } else {
                    return false;
                }
            }
        } else {
            if char == '"' && prev_char != '\\' {
                is_in_string = false;
            }
        }
        prev_char = char;
    }

    // Make sure all brackets matched up
    if !bracket_stack.is_empty() {
        return false;
    }

    return true;
}

fn get_json(file_name: Option<String>) -> io::Result<String> {
    // Providing both file_name and stdin will just process the file.
    let mut json = String::new();
    match file_name {
        Some(file_name) => {
            let mut reader = open_file(&file_name)?;
            let _ = reader.read_to_string(&mut json);
        }
        None => {
            /* If no input is provided, user is able to type in their own input then press Ctrl+D (sometimes twice). */
            let _ = io::stdin().read_to_string(&mut json);
        }
    }
    Ok(json)
}

fn open_file(file_name: &str) -> io::Result<BufReader<File>> {
    let file = File::open(file_name)?;
    Ok(BufReader::new(file))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_valid_json() -> io::Result<()> {
        let json = get_json(Some("test-json/step1/valid.json".into()))?;
        assert_eq!(parse_json(&json), true);
        Ok(())
    }

    #[test]
    fn test_simple_invalid_json() -> io::Result<()> {
        let json = get_json(Some("test-json/step1/invalid.json".into()))?;
        assert_eq!(parse_json(&json), false);
        Ok(())
    }
}
