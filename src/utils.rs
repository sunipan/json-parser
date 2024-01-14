use regex::Regex;
use std::{
    char,
    fs::File,
    io::{self, BufReader, Read},
};

pub enum ValueType {
    String,
    Number,
    Object,
    Array,
    True,
    False,
    Null,
    Error,
}

pub fn is_escaped_char_valid(char: char) -> bool {
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

pub fn is_string_valid(value: &String) -> bool {
    let value = value.trim();
    // let number_regex = Regex::new(r"-?\d+(\.\d+)?([eE][-+]?\d+)?").unwrap();
    let string_regex =
        Regex::new(r#"^"\s*((?:\\["\\\/bfnrt]|\\u[a-fA-F0-9]{4}|[^"\\])*)\s*"$"#).unwrap();
    return string_regex.is_match(&value);
}

pub fn print_err(msg: &str) {
    eprintln!("{}", msg);
}

pub fn is_number(value: &str) -> bool {
    let re = Regex::new(r"^-?(0|[1-9]\d*)(\.\d+)?([eE][+-]?\d+)?$").unwrap();
    return re.is_match(value);
}

pub fn get_value_type(value: &str) -> ValueType {
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

pub fn get_json(file_name: Option<String>) -> io::Result<String> {
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

pub fn open_file(file_name: &str) -> io::Result<BufReader<File>> {
    let file = File::open(file_name)?;
    Ok(BufReader::new(file))
}
