use crate::parser::ast::Expression;
use crate::parser::parser::parse_program;
use crate::runtime::runtime::Runtime;
use regex::Regex;
use std::fs;
use tiny_http::{Response, Server};

pub fn run(folder: &str) {
    let mut routes: Vec<Route> = Vec::new();

    let entries = match fs::read_dir(folder) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("[webcore] Failed to read directory '{}': {}", folder, e);
            return;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("vx") {
            continue;
        }

        let code = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[webcore] Failed to read '{}': {}", path.display(), e);
                continue;
            }
        };

        let statements = parse_program(&code);
        let mut runtime = Runtime::new();
        runtime.execute(statements);

        let default_path = format!(
            "/{}",
            path.file_stem().unwrap_or_default().to_string_lossy()
        );
        let path_value = runtime
            .get_variable("path")
            .and_then(|e| match e {
                Expression::StringLiteral(s) => Some(s),
                _ => None,
            })
            .unwrap_or(default_path);

        let method_value = runtime
            .get_variable("method")
            .and_then(|e| match e {
                Expression::StringLiteral(s) => Some(s.to_uppercase()),
                _ => None,
            })
            .unwrap_or_else(|| "GET".to_string());

        if !runtime.has_function("request") {
            eprintln!(
                "[webcore] File '{}' does not define exported function 'request', skipping",
                path.display()
            );
            continue;
        }

        routes.push(Route {
            path: path_value,
            method: method_value,
            runtime,
        });
    }

    if routes.is_empty() {
        eprintln!("[webcore] No .vx endpoints found in '{}'.", folder);
        return;
    }

    println!("WebCore server running on http://localhost:8080 🦀");
    for r in &routes {
        println!("  {} {}", r.method, r.path);
    }

    let server = match Server::http("0.0.0.0:8080") {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[webcore] Failed to bind HTTP server: {}", e);
            return;
        }
    };

    for request in server.incoming_requests() {
        let url = request.url().to_string();
        let method = request.method().as_str().to_uppercase();

        if let Some(route) = routes.iter_mut().find(|r| r.matches(&url, &method)) {
            let arg_opt = route.extract_arg(&url);
            let mut args = Vec::new();
            if let Some(a) = arg_opt {
                args.push(Expression::StringLiteral(a));
            }

            let body = route
                .runtime
                .call_function("request", args)
                .and_then(|e| match e {
                    Expression::StringLiteral(s) => Some(s),
                    _ => None,
                })
                .unwrap_or_else(|| "".to_string());

            let _ = request.respond(Response::from_string(body));
        } else {
            let _ = request.respond(Response::from_string("Not Found").with_status_code(404));
        }
    }
}

struct Route {
    path: String,
    method: String,
    runtime: Runtime,
}

impl Route {
    fn matches(&self, url: &str, method: &str) -> bool {
        if self.method != method {
            return false;
        }
        if self.path.contains('{') {
            let re = self.to_regex();
            re.is_match(url)
        } else {
            self.path == url
        }
    }

    fn extract_arg(&self, url: &str) -> Option<String> {
        if self.path.contains('{') {
            let re = self.to_regex();
            if let Some(cap) = re.captures(url).and_then(|c| c.get(1)) {
                return Some(cap.as_str().to_string());
            }
        }
        None
    }

    fn to_regex(&self) -> Regex {
        let mut pattern = regex::escape(&self.path);
        let placeholder = Regex::new(r"\\\{[^\\]+\\\}").unwrap();
        pattern = placeholder.replace_all(&pattern, "([^/]+)").into_owned();
        Regex::new(&format!("^{}$", pattern)).expect("Invalid regex")
    }
}
