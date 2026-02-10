use crate::parser::ast::{Expression, Statement};
use crate::runtime::runtime::Runtime;
use rustc_hash::FxHashMap as HashMap;
use std::sync::Mutex;
use std::thread::JoinHandle;
use std::time::{SystemTime, UNIX_EPOCH};

lazy_static::lazy_static! {
    static ref THREADS: Mutex<HashMap<String, JoinHandle<Option<Expression>>>> = Mutex::new(HashMap::default());
    static ref RESULTS: Mutex<HashMap<String, Option<Expression>>> = Mutex::new(HashMap::default());
    static ref FUNCTIONS_SNAPSHOT: Mutex<HashMap<String, (Vec<String>, Vec<Statement>, bool)>> = Mutex::new(HashMap::default());
    static ref MODULES_SNAPSHOT: Mutex<HashMap<String, HashMap<String, (Vec<String>, Vec<Statement>, bool)>>> = Mutex::new(HashMap::default());
    static ref MODULE_CACHE_SNAPSHOT: Mutex<HashMap<String, HashMap<String, (Vec<String>, Vec<Statement>, bool)>>> = Mutex::new(HashMap::default());
}

pub fn update_functions_snapshot(map: HashMap<String, (Vec<String>, Vec<Statement>, bool)>) {
    let mut guard = FUNCTIONS_SNAPSHOT.lock().unwrap();
    *guard = map;
}

pub fn update_modules_snapshot(map: HashMap<String, HashMap<String, (Vec<String>, Vec<Statement>, bool)>>) {
    let mut guard = MODULES_SNAPSHOT.lock().unwrap();
    *guard = map;
}

pub fn update_module_cache_snapshot(map: HashMap<String, HashMap<String, (Vec<String>, Vec<Statement>, bool)>>) {
    let mut guard = MODULE_CACHE_SNAPSHOT.lock().unwrap();
    *guard = map;
}

fn thread_spawn(args: Vec<Expression>) -> Option<Expression> {
    if args.is_empty() {
        eprintln!("thread_spawn requires at least a function name");
        return None;
    }

    let func_name = match &args[0] {
        Expression::StringLiteral(s) => s.clone(),
        _ => {
            eprintln!("thread_spawn first argument must be a string function name");
            return None;
        }
    };

    // Tail args passed to the target function
    let call_args: Vec<Expression> = args.into_iter().skip(1).collect();

    // Snapshot current functions/modules
    let funcs = FUNCTIONS_SNAPSHOT.lock().unwrap().clone();
    let mods = MODULES_SNAPSHOT.lock().unwrap().clone();
    let mod_cache = MODULE_CACHE_SNAPSHOT.lock().unwrap().clone();

    // Generate a simple thread id
    let tid = format!(
        "t{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    );

    let handle = std::thread::spawn(move || {
        let mut rt = Runtime::new();
        // Inject snapshots into the new runtime
        rt.set_functions_snapshot(funcs);
        rt.set_modules_snapshot(mods, mod_cache);
        // Call the function; name may be module-qualified
        rt.call_function(&func_name, call_args)
    });

    THREADS.lock().unwrap().insert(tid.clone(), handle);
    Some(Expression::StringLiteral(tid))
}

fn thread_join(args: Vec<Expression>) -> Option<Expression> {
    if args.len() != 1 {
        eprintln!("thread_join requires exactly one argument: thread id");
        return None;
    }

    let tid = match &args[0] {
        Expression::StringLiteral(s) => s.clone(),
        _ => {
            eprintln!("thread_join argument must be a string thread id");
            return None;
        }
    };

    // Try to take and join the thread if still running
    if let Some(handle) = THREADS.lock().unwrap().remove(&tid) {
        match handle.join() {
            Ok(result) => {
                RESULTS.lock().unwrap().insert(tid.clone(), result.clone());
                return result;
            }
            Err(_) => {
                eprintln!("thread '{}' panicked during execution", tid);
                RESULTS.lock().unwrap().insert(tid.clone(), None);
                return None;
            }
        }
    }

    // Otherwise, return a stored result if already joined
    if let Some(result) = RESULTS.lock().unwrap().remove(&tid) {
        return result;
    }

    // Unknown id
    eprintln!("thread '{}' not found", tid);
    None
}

fn thread_status(args: Vec<Expression>) -> Option<Expression> {
    if args.len() != 1 {
        eprintln!("thread_status requires exactly one argument: thread id");
        return None;
    }

    let tid = match &args[0] {
        Expression::StringLiteral(s) => s.clone(),
        _ => {
            eprintln!("thread_status argument must be a string thread id");
            return None;
        }
    };

    let threads = THREADS.lock().unwrap();
    if threads.contains_key(&tid) {
        return Some(Expression::StringLiteral("running".to_string()));
    }
    drop(threads);

    let results = RESULTS.lock().unwrap();
    if results.contains_key(&tid) {
        return Some(Expression::StringLiteral("done".to_string()));
    }

    Some(Expression::StringLiteral("unknown".to_string()))
}

pub fn thread_functions() -> Vec<(&'static str, fn(Vec<Expression>) -> Option<Expression>)> {
    vec![
        ("thread_spawn", thread_spawn as fn(Vec<Expression>) -> Option<Expression>),
        ("thread_join", thread_join as fn(Vec<Expression>) -> Option<Expression>),
        ("thread_status", thread_status as fn(Vec<Expression>) -> Option<Expression>),
    ]
}