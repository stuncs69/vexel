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

#[cfg(test)]
mod tests {
    use super::fs_functions;
    use crate::parser::ast::Expression;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn fs_fn(name: &str) -> fn(Vec<Expression>) -> Option<Expression> {
        fs_functions()
            .into_iter()
            .find(|(n, _)| *n == name)
            .map(|(_, f)| f)
            .expect("missing fs function")
    }

    fn temp_path(name: &str) -> String {
        let mut path = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before unix epoch")
            .as_nanos();
        path.push(format!("vexel_fs_test_{}_{}_{}", name, std::process::id(), nanos));
        path.to_string_lossy().to_string()
    }

    #[test]
    fn write_read_append_exists_and_delete_work() {
        let write_file = fs_fn("write_file");
        let read_file = fs_fn("read_file");
        let append_file = fs_fn("append_file");
        let file_exists = fs_fn("file_exists");
        let delete_file = fs_fn("delete_file");

        let path = temp_path("file");

        assert!(matches!(
            file_exists(vec![Expression::StringLiteral(path.clone())]),
            Some(Expression::Boolean(false))
        ));
        assert!(write_file(vec![
            Expression::StringLiteral(path.clone()),
            Expression::StringLiteral("a".to_string())
        ])
        .is_some());
        assert!(append_file(vec![
            Expression::StringLiteral(path.clone()),
            Expression::StringLiteral("b".to_string())
        ])
        .is_some());
        assert!(matches!(
            read_file(vec![Expression::StringLiteral(path.clone())]),
            Some(Expression::StringLiteral(content)) if content == "ab"
        ));
        assert!(matches!(
            file_exists(vec![Expression::StringLiteral(path.clone())]),
            Some(Expression::Boolean(true))
        ));
        assert!(delete_file(vec![Expression::StringLiteral(path)]).is_some());
    }

    #[test]
    fn create_dir_and_list_dir_work() {
        let create_dir = fs_fn("create_dir");
        let list_dir = fs_fn("list_dir");
        let write_file = fs_fn("write_file");

        let dir = temp_path("dir");
        let file_path = format!("{}/x.txt", dir);

        assert!(create_dir(vec![Expression::StringLiteral(dir.clone())]).is_some());
        assert!(write_file(vec![
            Expression::StringLiteral(file_path),
            Expression::StringLiteral("x".to_string())
        ])
        .is_some());

        assert!(matches!(
            list_dir(vec![Expression::StringLiteral(dir)]),
            Some(Expression::Array(items)) if !items.is_empty()
        ));
    }
}
