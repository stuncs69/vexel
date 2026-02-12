use crate::parser::ast::Expression;
use rustc_hash::FxHashMap as HashMap;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

lazy_static::lazy_static! {
    static ref CHANNEL_SENDERS: Mutex<HashMap<String, Sender<Expression>>> = Mutex::new(HashMap::default());
    static ref CHANNEL_RECEIVERS: Mutex<HashMap<String, Receiver<Expression>>> = Mutex::new(HashMap::default());
}

fn next_channel_id() -> String {
    format!(
        "ch{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    )
}

fn thread_channel(args: Vec<Expression>) -> Option<Expression> {
    if !args.is_empty() {
        return None;
    }

    let id = next_channel_id();
    let (sender, receiver) = mpsc::channel::<Expression>();
    CHANNEL_SENDERS.lock().ok()?.insert(id.clone(), sender);
    CHANNEL_RECEIVERS.lock().ok()?.insert(id.clone(), receiver);

    Some(Expression::StringLiteral(id))
}

fn thread_send(args: Vec<Expression>) -> Option<Expression> {
    if args.len() != 2 {
        return None;
    }

    let channel_id = match &args[0] {
        Expression::StringLiteral(s) => s.clone(),
        _ => return None,
    };

    let value = args[1].clone();
    let guard = CHANNEL_SENDERS.lock().ok()?;
    let sender = guard.get(&channel_id)?;
    sender.send(value).ok()?;
    Some(Expression::Null)
}

fn thread_recv(args: Vec<Expression>) -> Option<Expression> {
    if args.len() != 1 {
        return None;
    }

    let channel_id = match &args[0] {
        Expression::StringLiteral(s) => s.clone(),
        _ => return None,
    };

    let guard = CHANNEL_RECEIVERS.lock().ok()?;
    let receiver = guard.get(&channel_id)?;
    receiver.recv().ok()
}

fn thread_close(args: Vec<Expression>) -> Option<Expression> {
    if args.len() != 1 {
        return None;
    }

    let channel_id = match &args[0] {
        Expression::StringLiteral(s) => s.clone(),
        _ => return None,
    };

    CHANNEL_SENDERS.lock().ok()?.remove(&channel_id)?;
    CHANNEL_RECEIVERS.lock().ok()?.remove(&channel_id)?;

    Some(Expression::Null)
}

pub fn thread_functions() -> Vec<(&'static str, fn(Vec<Expression>) -> Option<Expression>)> {
    vec![
        (
            "thread_channel",
            thread_channel as fn(Vec<Expression>) -> Option<Expression>,
        ),
        (
            "thread_send",
            thread_send as fn(Vec<Expression>) -> Option<Expression>,
        ),
        (
            "thread_recv",
            thread_recv as fn(Vec<Expression>) -> Option<Expression>,
        ),
        (
            "thread_close",
            thread_close as fn(Vec<Expression>) -> Option<Expression>,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::thread_functions;
    use crate::parser::ast::Expression;

    fn thread_fn(name: &str) -> fn(Vec<Expression>) -> Option<Expression> {
        thread_functions()
            .into_iter()
            .find(|(n, _)| *n == name)
            .map(|(_, f)| f)
            .expect("missing thread function")
    }

    #[test]
    fn channel_send_and_recv_roundtrip() {
        let channel = thread_fn("thread_channel");
        let send = thread_fn("thread_send");
        let recv = thread_fn("thread_recv");

        let Some(Expression::StringLiteral(id)) = channel(vec![]) else {
            panic!("thread_channel should return a channel id");
        };

        assert!(send(vec![
            Expression::StringLiteral(id.clone()),
            Expression::Number(42)
        ])
        .is_some());

        assert!(matches!(
            recv(vec![Expression::StringLiteral(id)]),
            Some(Expression::Number(42))
        ));
    }

    #[test]
    fn recv_unknown_channel_returns_none() {
        let recv = thread_fn("thread_recv");
        assert!(recv(vec![Expression::StringLiteral("missing".to_string())]).is_none());
    }
}
