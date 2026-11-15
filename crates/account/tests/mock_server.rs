//! Needs the mock site: run `scripts/mock-account-server.sh`, then `cargo test -p account -- --ignored`.

use std::io::{Read, Write};
use std::net::TcpStream;

use account::client::{self, ClientError};
use account::pkce::Pkce;
use account::schedule::CheckOutcome;
use account::status::Status;
use account::stored::StoredAccount;

const DEVICE_HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

/// Plays the browser: submits the mock site's email form and reads the code from the redirect to the app.
fn login_code(pkce: &Pkce) -> String {
    let address = client::site_url().trim_start_matches("http://").to_owned();
    let form = format!(
        "email=test{}%40example.com&port=51234&challenge={}&state={}&device_hash={DEVICE_HASH}&name=Test+PC",
        std::process::id(),
        pkce.challenge,
        pkce.state
    );
    let request = format!(
        "POST /device/login HTTP/1.1\r\nHost: {address}\r\nContent-Type: application/x-www-form-urlencoded\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n{form}",
        form.len()
    );
    let mut stream = TcpStream::connect(&address).expect("the mock server is running");
    stream.write_all(request.as_bytes()).expect("request sent");
    let mut response = String::new();
    stream.read_to_string(&mut response).expect("response read");
    let location = response
        .lines()
        .find_map(|line| line.strip_prefix("Location: "))
        .unwrap_or_else(|| panic!("no redirect in: {response}"));
    let query = location.split_once("code=").expect("a code in the redirect").1;
    query.split('&').next().expect("code value").to_owned()
}

#[test]
#[ignore = "needs the mock server from scripts/mock-account-server.sh"]
fn claims_and_checks_against_the_mock_server() {
    let pkce = Pkce::new().expect("cng works");
    assert_eq!(client::claim("not-a-code", &pkce.verifier), Err(ClientError::InvalidCode));

    let claimed = client::claim(&login_code(&pkce), &pkce.verifier).expect("claimed");
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).expect("clock").as_secs() as i64;
    let account = StoredAccount {
        device_id: Some(claimed.device_id.clone()),
        device_secret: Some(claimed.device_secret.clone()),
        token: Some(claimed.token),
        policy: Some(claimed.policy),
        last_check_at: Some(now),
    };
    assert_ne!(account.access(DEVICE_HASH, now).status, Status::Guest);

    let renewed = client::check(&claimed.device_id, &claimed.device_secret);
    assert!(matches!(renewed, CheckOutcome::Renewed { token: Some(_), .. }));
    assert!(matches!(client::fetch_policy(), CheckOutcome::Renewed { token: None, .. }));
    assert_eq!(client::check(&claimed.device_id, "wrong-secret"), CheckOutcome::UnknownDevice);

    client::sign_out(&claimed.device_id, &claimed.device_secret);
    assert_eq!(client::check(&claimed.device_id, &claimed.device_secret), CheckOutcome::SignedOut);
}
