use clap::Parser;
use regex::Regex;
use std::{
    char,
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
enum Token {
    String(String),
    Number(f64),
    Bool(bool),
    Null,
    OpenCurly,
    CloseCurly,
    OpenSquare,
    CloseSquare,
    Colon,
    Comma,
    Escape,
}

enum ValueType {
    String,
    Number,
    Object,
    Array,
    True,
    False,
    Null,
    Error,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let json = get_json(args.file_name)?;
    let is_valid = parse_json(&json);

    if is_valid {
        println!("{}, VALID", 0);
    } else {
        println!("{}, INVALID", 1)
    }
    Ok(())
}

fn parse_json(json: &String) -> bool {
    if json.is_empty() {
        print_err("JSON is empty");
        return false;
    }

    // let mut token_vec: Vec<Token> = Vec::new();
    // let mut is_in_hexadecimal = false;
    let mut did_just_exit_value = false;
    let mut should_check_after_exiting_value = false;
    let mut is_in_string = false;
    let mut is_in_value = false;
    let mut brack_stack: Vec<char> = Vec::new();
    let mut current_string = String::new();
    let mut prev_char: char = '\0';

    let chars = json.chars();

    for char in chars {
        // Checks if key is bad
        if did_just_exit_value
            && should_check_after_exiting_value
            && char != ' '
            && char != '"'
            && char != '}'
            && char != ','
            && char != '\n'
        {
            print_err("Invalid char after exiting value");
            return false;
        } else if char == '"' || char == '}' {
            should_check_after_exiting_value = false;
        }
        print!("{}, ", char);
        // Both is_in_string and is_in_value can be true
        if is_in_string {
            current_string.push(char);
            if is_in_string {
                if char == '"' && prev_char != '\\' {
                    is_in_string = false;
                }
                if prev_char == '\\' && !is_escaped_char_valid(char) {
                    print_err("Invalid escaped character");
                    return false;
                }
            }
        } else if is_in_value && !is_in_string {
            let mut should_concat = true;
            if char == ',' {
                is_in_value = false;
                did_just_exit_value = true;
                should_check_after_exiting_value = true;
                should_concat = false;
            } else if char == '}' {
                // Verify bracket closures
                if brack_stack.len() > 0 && brack_stack[brack_stack.len() - 1] == '{' {
                    brack_stack.pop();
                } else {
                    print_err("No matching end curly brace");
                    return false;
                }
                if is_in_value {
                    // Works as comma in this case
                    is_in_value = false;
                    did_just_exit_value = true;
                    should_check_after_exiting_value = true;
                }
                should_concat = false;
            }

            if should_concat {
                current_string.push(char);
            } else {
                let value_trimmed = current_string.trim();
                let value_type = get_value_type(&value_trimmed);
                match value_type {
                    ValueType::String => {
                        if !is_string_valid(&current_string) {
                            print_err("Invalid value string");
                            return false;
                        }
                    }
                    ValueType::Error => {
                        print_err("Invalid value");
                        return false;
                    }
                    _ => {}
                };
                current_string.clear();
            }
        } else {
            if !current_string.is_empty() {
                // Can be both value and key
                if !is_in_value && !is_string_valid(&current_string) {
                    print_err("Invalid string");
                    return false;
                }
                current_string.clear();
            }
            if char == '{' {
                brack_stack.push(char);
            } else if char == '"' {
                is_in_string = true;
                current_string.push('"');
            } else if char == ':' {
                is_in_value = true;
            } else if char == '}' {
                if prev_char == ',' {
                    print_err("Invalid trailing comma");
                    return false;
                }
                // Verify bracket closures
                if brack_stack.len() > 0 && brack_stack[brack_stack.len() - 1] == '{' {
                    brack_stack.pop();
                } else {
                    print_err("No matching end curly brace, not in value.");
                    return false;
                }
            }
        }
        prev_char = char;
    }

    // Make sure all brackets matched up
    if !brack_stack.is_empty() || is_in_string || is_in_value {
        print_err("Either bracket stack not empty, still in string or still in value");
        println!("{:?}, {}, {}", brack_stack, is_in_string, is_in_value);
        return false;
    }

    return true;
}

fn is_escaped_char_valid(char: char) -> bool {
    if char != '"'
        && char != '\\'
        && char != '/'
        && char != 'b'
        && char != 'f'
        && char != 'n'
        && char != 'r'
        && char != 't'
    // implement \u + 4 hex digits
    {
        return false;
    }
    return true;
}

fn is_string_valid(value: &String) -> bool {
    let value = value.trim();
    // let number_regex = Regex::new(r"-?\d+(\.\d+)?([eE][-+]?\d+)?").unwrap();
    let string_regex =
        Regex::new(r#"^"\s*((?:\\["\\\/bfnrt]|\\u[a-fA-F0-9]{4}|[^"\\])*)\s*"$"#).unwrap();
    return string_regex.is_match(&value);
}

fn print_err(msg: &str) {
    eprintln!("{}", msg);
}

fn is_number(value: &str) -> bool {
    let re = Regex::new(r"^-?(0|[1-9]\d*)(\.\d+)?([eE][+-]?\d+)?$").unwrap();
    return re.is_match(value);
}

fn get_value_type(value: &str) -> ValueType {
    if value.starts_with('"') {
        return ValueType::String;
    } else if is_number(&value) {
        return ValueType::Number;
    } else if value.starts_with('[') {
        return ValueType::Array;
    } else if value.starts_with('{') {
        return ValueType::Object;
    } else if value.eq("true") {
        return ValueType::True;
    } else if value.eq("false") {
        return ValueType::False;
    } else if value.eq("null") {
        return ValueType::Null;
    } else {
        return ValueType::Error;
    }
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
    fn test_step1_valid_json() -> io::Result<()> {
        let json = get_json(Some("test-json/step1/valid.json".into()))?;
        assert_eq!(parse_json(&json), true);
        Ok(())
    }

    #[test]
    fn test_step1_invalid_json() -> io::Result<()> {
        let json = get_json(Some("test-json/step1/invalid.json".into()))?;
        assert_eq!(parse_json(&json), false);
        Ok(())
    }

    #[test]
    fn test_step2_valid_json() -> io::Result<()> {
        let json = get_json(Some("test-json/step2/valid.json".into()))?;
        assert_eq!(parse_json(&json), true);
        Ok(())
    }

    #[test]
    fn test_step2_valid2_json() -> io::Result<()> {
        let json = get_json(Some("test-json/step2/valid2.json".into()))?;
        assert_eq!(parse_json(&json), true);
        Ok(())
    }

    #[test]
    fn test_step2_invalid_json() -> io::Result<()> {
        let json = get_json(Some("test-json/step2/invalid.json".into()))?;
        assert_eq!(parse_json(&json), false);
        Ok(())
    }

    #[test]
    fn test_step2_invalid2_json() -> io::Result<()> {
        let json = get_json(Some("test-json/step2/invalid2.json".into()))?;
        assert_eq!(parse_json(&json), false);
        Ok(())
    }

    #[test]
    fn test_step3_valid_json() -> io::Result<()> {
        let json = get_json(Some("test-json/step3/valid.json".into()))?;
        assert_eq!(parse_json(&json), true);
        Ok(())
    }

    #[test]
    fn test_step3_invalid_json() -> io::Result<()> {
        let json = get_json(Some("test-json/step3/invalid.json".into()))?;
        assert_eq!(parse_json(&json), false);
        Ok(())
    }

    // #[test]
    // fn test_step4_valid_json() -> io::Result<()> {
    //     let json = get_json(Some("test-json/step4/valid.json".into()))?;
    //     assert_eq!(parse_json(&json), true);
    //     Ok(())
    // }

    // #[test]
    // fn test_step4_valid2_json() -> io::Result<()> {
    //     let json = get_json(Some("test-json/step4/valid2.json".into()))?;
    //     assert_eq!(parse_json(&json), true);
    //     Ok(())
    // }

    // #[test]
    // fn test_step4_invalid_json() -> io::Result<()> {
    //     let json = get_json(Some("test-json/step4/invalid.json".into()))?;
    //     assert_eq!(parse_json(&json), false);
    //     Ok(())
    // }
}
