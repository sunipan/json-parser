mod utils;
use clap::Parser;
use std::{char, io};
use utils::{
    get_json, get_value_type, is_escaped_char_valid, is_string_valid, print_err, ValueType,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // Optional file to run operations on
    file_name: Option<String>,
}

/* What if we first parsed all characters into an array of JSON tokens without unnecessary whitespaces.
This could be memory intensive if a JSON file is massive, but would allow for easier parsing.
Could create one bigger JSON parser, or a bunch of tiny JSON parsers for each type of value and
use them recursively (especially the object parser)!
Diagram of thought: https://excalidraw.com/#json=OnoN3rfV0x-tLy29MHNAV,m_9HV8Xr5Zjw5xgksa0dPw */

/* What if we made parse_json recursive where we can pass the chars iterator to the next
function which continues the iteration when we encounter an object? Once it's done the object
it returns either true or false. If false, we return false, if true, we keep going with the iterator
until the there is no more to parse. Diagram of thought: https://excalidraw.com/#json=dM_uO3bgEwExHfYPO-YQW,mjpOY2pv7bJVtYvhQ1jt5w */

enum Token {
    Value(ValueType),
    Key(String),
    OpenCurly,
    CloseCurly,
    OpenSquare,
    CloseSquare,
    Colon,
    Comma,
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

fn parse_json_v2(json: &String) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut current_string = String::new();
    let mut prev_char: char = '\0';
    let mut is_in_string = false;

    for char in json.chars() {
        if is_in_string {
            current_string.push(char);
            if prev_char != '\\' && char == '"' {
                is_in_string = false;
            }
        } else {
            if char == '{' {
                tokens.push(Token::OpenCurly);
            } else if char == '}' {
                tokens.push(Token::CloseCurly);
            } else if char == ',' {
                tokens.push(Token::Comma);
            } else if char == ':' {
                tokens.push(Token::Colon);
            } else if char == '"' {
                current_string.push(char);
                is_in_string = true;
            }
        }
        prev_char = char;
    }

    return tokens;
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
