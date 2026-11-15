//! The loopback port the site redirects the browser to: it answers every connection on `127.0.0.1` until the site's
//! callback for this sign-in arrives or the wait is stopped, then it closes.

use std::{
    io::{self, Read, Write},
    net::{Ipv4Addr, Shutdown, SocketAddr, TcpListener, TcpStream},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use super::{Callback, parse_callback};

/// How long one connection gets to send its request; a browser's idle preconnect gives up its turn after this.
const READ_LIMIT: Duration = Duration::from_secs(2);
/// How long the answered connection is drained after the answer, so the browser reads the page instead of a reset.
const DRAIN_LIMIT: Duration = Duration::from_millis(200);
const STOP_CONNECT_LIMIT: Duration = Duration::from_secs(1);
/// A request head longer than this is not the site's callback.
const MAX_REQUEST_BYTES: usize = 8 * 1024;

/// The waiting end, moved to a background thread; `wait` blocks until the callback or a stop.
pub struct Listener {
    listener: TcpListener,
    finished: Arc<AtomicBool>,
}

/// Ends a wait from the UI thread: the Cancel button or the five-minute limit.
#[derive(Clone)]
pub struct Stopper {
    port: u16,
    finished: Arc<AtomicBool>,
}

/// How a wait ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Waited {
    /// A code or a cancel carrying this sign-in's state.
    Answered(Callback),
    /// The Cancel button or the time limit.
    Stopped,
    /// The port stopped accepting connections.
    Failed,
}

impl Listener {
    /// Listens on a free port of `127.0.0.1`.
    pub fn open() -> io::Result<(Listener, Stopper)> {
        let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))?;
        let port = listener.local_addr()?.port();
        let finished = Arc::new(AtomicBool::new(false));
        Ok((Listener { listener, finished: finished.clone() }, Stopper { port, finished }))
    }

    /// Answers every connection until one carries a code or a cancel for `expected_state`, which gets `page`; anything
    /// else (a preconnect, `/favicon.ico`, another state) gets a 404 and the wait goes on. The port closes on return.
    pub fn wait(self, expected_state: &str, page: &str) -> Waited {
        loop {
            let Ok((mut stream, _)) = self.listener.accept() else {
                self.finished.store(true, Ordering::SeqCst);
                return Waited::Failed;
            };
            // A stop sets the flag before it connects, so this connection is the stop's own wake-up.
            if self.finished.load(Ordering::SeqCst) {
                return Waited::Stopped;
            }
            let callback = read_request_line(&mut stream).map(|line| parse_callback(&line, expected_state));
            let Some(callback @ (Callback::Code(_) | Callback::Cancelled)) = callback else {
                answer(&mut stream, NOT_FOUND);
                continue;
            };
            // Whoever sets the flag first wins: a stop that came in while this request was read keeps its meaning.
            if self.finished.swap(true, Ordering::SeqCst) {
                return Waited::Stopped;
            }
            answer(&mut stream, &response(page));
            return Waited::Answered(callback);
        }
    }
}

impl Stopper {
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Wakes the waiting thread with a connection of its own, unless the wait has already ended.
    pub fn stop(&self) {
        if !self.finished.swap(true, Ordering::SeqCst) {
            let address = SocketAddr::from((Ipv4Addr::LOCALHOST, self.port));
            let _ = TcpStream::connect_timeout(&address, STOP_CONNECT_LIMIT);
        }
    }
}

/// Reads the request head up to its blank line and returns its first line; `None` for silence, a close or junk.
fn read_request_line(stream: &mut TcpStream) -> Option<String> {
    stream.set_read_timeout(Some(READ_LIMIT)).ok()?;
    let mut received = Vec::new();
    let mut buffer = [0u8; 1024];
    while !received.windows(4).any(|window| window == b"\r\n\r\n") {
        let count = stream.read(&mut buffer).ok()?;
        if count == 0 || received.len() + count > MAX_REQUEST_BYTES {
            return None;
        }
        received.extend_from_slice(&buffer[..count]);
    }
    let text = String::from_utf8_lossy(&received);
    text.split("\r\n").next().map(str::to_owned)
}

const NOT_FOUND: &str = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";

/// Sends the whole answer, closes the sending side and drains briefly, so the browser is not reset mid-page.
fn answer(stream: &mut TcpStream, text: &str) {
    let _ = stream.write_all(text.as_bytes());
    let _ = stream.flush();
    let _ = stream.shutdown(Shutdown::Write);
    let _ = stream.set_read_timeout(Some(DRAIN_LIMIT));
    let mut buffer = [0u8; 1024];
    while matches!(stream.read(&mut buffer), Ok(count) if count > 0) {}
}

/// A complete HTTP/1.1 answer with `page` as a small HTML document; the connection closes after it.
fn response(page: &str) -> String {
    let body = format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>Notch</title></head>\
         <body style=\"font-family:system-ui,sans-serif;text-align:center;margin-top:20vh\"><p>{}</p></body></html>",
        escape_html(page)
    );
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-store\r\n\
         Connection: close\r\n\r\n{body}",
        body.len()
    )
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    const CALLBACK: &str = "GET /callback?code=abc&state=st HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n";

    fn send(port: u16, request: &str) -> String {
        let mut stream = TcpStream::connect((Ipv4Addr::LOCALHOST, port)).expect("connect");
        stream.write_all(request.as_bytes()).expect("send");
        let mut answer = String::new();
        stream.read_to_string(&mut answer).expect("answer");
        answer
    }

    fn waiting(page: &'static str) -> (u16, std::thread::JoinHandle<Waited>, Stopper) {
        let (listener, stopper) = Listener::open().expect("listen");
        let port = stopper.port();
        (port, std::thread::spawn(move || listener.wait("st", page)), stopper)
    }

    #[test]
    fn the_browsers_callback_gets_the_page_and_its_code_is_returned() {
        let (port, waiter, _stopper) = waiting("You can close this tab");
        let answer = send(port, CALLBACK);
        assert_eq!(waiter.join().expect("waiter"), Waited::Answered(Callback::Code("abc".to_owned())));
        assert!(answer.starts_with("HTTP/1.1 200 OK") && answer.contains("You can close this tab"), "{answer}");
    }

    #[test]
    fn an_empty_connection_and_a_favicon_request_before_the_callback_still_sign_in() {
        let (port, waiter, _stopper) = waiting("page");
        drop(TcpStream::connect((Ipv4Addr::LOCALHOST, port)).expect("preconnect"));
        let favicon = send(port, "GET /favicon.ico HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n");
        assert!(favicon.starts_with("HTTP/1.1 404"), "{favicon}");
        send(port, CALLBACK);
        assert_eq!(waiter.join().expect("waiter"), Waited::Answered(Callback::Code("abc".to_owned())));
    }

    #[test]
    fn a_wrong_state_request_does_not_end_the_sign_in() {
        let (port, waiter, _stopper) = waiting("page");
        let refused = send(port, "GET /callback?code=evil&state=other HTTP/1.1\r\n\r\n");
        assert!(refused.starts_with("HTTP/1.1 404"), "{refused}");
        send(port, "GET /callback?error=cancelled&state=st HTTP/1.1\r\n\r\n");
        assert_eq!(waiter.join().expect("waiter"), Waited::Answered(Callback::Cancelled));
    }

    #[test]
    fn a_stop_ends_the_wait_without_a_callback() {
        let (_port, waiter, stopper) = waiting("page");
        stopper.stop();
        assert_eq!(waiter.join().expect("waiter"), Waited::Stopped);
    }

    #[test]
    fn the_port_is_closed_after_the_callback() {
        let (port, waiter, _stopper) = waiting("page");
        send(port, CALLBACK);
        waiter.join().expect("waiter");
        let address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
        assert!(TcpStream::connect_timeout(&address, STOP_CONNECT_LIMIT).is_err());
    }

    #[test]
    fn the_answer_page_escapes_its_text_and_states_its_length() {
        let answer = response("a <b> & \"c\"");
        assert!(answer.contains("a &lt;b&gt; &amp; &quot;c&quot;"));
        let (head, body) = answer.split_once("\r\n\r\n").expect("head and body");
        assert!(head.contains(&format!("Content-Length: {}", body.len())));
    }
}
