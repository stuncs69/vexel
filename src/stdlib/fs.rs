use crate::parser::ast::Expression;
use std::fs;
use std::io::Write;

pub fn fs_functions() -> Vec<(&'static str, fn(Vec<Expression>) -> Option<Expression>)> {
    vec![
        ("read_file", |args: Vec<Expression>| {
            if args.len() == 1 {
                if let Expression::StringLiteral(path) = &args[0] {
                    fs::read_to_string(path)
                        .ok()
                        .map(|content| Expression::StringLiteral(content))
                } else {
                    None
                }
            } else {
                None
            }
        }),
        ("write_file", |args: Vec<Expression>| {
            if args.len() == 2 {
                if let (Expression::StringLiteral(path), Expression::StringLiteral(content)) =
                    (&args[0], &args[1])
                {
                    match fs::File::create(path) {
                        Ok(mut file) => match file.write_all(content.as_bytes()) {
                            Ok(_) => Some(Expression::Null),
                            Err(_) => None,
                        },
                        Err(_) => None,
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }),
        ("append_file", |args: Vec<Expression>| {
            if args.len() == 2 {
                if let (Expression::StringLiteral(path), Expression::StringLiteral(content)) =
                    (&args[0], &args[1])
                {
                    match fs::OpenOptions::new().append(true).create(true).open(path) {
                        Ok(mut file) => match file.write_all(content.as_bytes()) {
                            Ok(_) => Some(Expression::Null),
                            Err(_) => None,
                        },
                        Err(_) => None,
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }),
        ("file_exists", |args: Vec<Expression>| {
            if args.len() == 1 {
                if let Expression::StringLiteral(path) = &args[0] {
                    Some(Expression::Boolean(fs::metadata(path).is_ok()))
                } else {
                    None
                }
            } else {
                None
            }
        }),
        ("delete_file", |args: Vec<Expression>| {
            if args.len() == 1 {
                if let Expression::StringLiteral(path) = &args[0] {
                    match fs::remove_file(path) {
                        Ok(_) => Some(Expression::Null),
                        Err(_) => None,
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }),
        ("rename_file", |args: Vec<Expression>| {
            if args.len() == 2 {
                if let (Expression::StringLiteral(from), Expression::StringLiteral(to)) =
                    (&args[0], &args[1])
                {
                    match fs::rename(from, to) {
                        Ok(_) => Some(Expression::Null),
                        Err(_) => None,
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }),
        ("create_dir", |args: Vec<Expression>| {
            if args.len() == 1 {
                if let Expression::StringLiteral(path) = &args[0] {
                    match fs::create_dir_all(path) {
                        Ok(_) => Some(Expression::Null),
                        Err(_) => None,
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }),
        ("list_dir", |args: Vec<Expression>| {
            if args.len() == 1 {
                if let Expression::StringLiteral(path) = &args[0] {
                    match fs::read_dir(path) {
                        Ok(entries) => {
                            let mut names = Vec::new();
                            for entry in entries {
                                if let Ok(dir_entry) = entry {
                                    if let Some(name) = dir_entry.file_name().to_str() {
                                        names.push(Expression::StringLiteral(name.to_string()));
                                    }
                                }
                            }
                            Some(Expression::Array(names))
                        }
                        Err(_) => None,
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }),
    ]
}
