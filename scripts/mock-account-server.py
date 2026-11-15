"""Mock of the Notch website's device API (routes 1-5 of the website contract), in memory.

Run: scripts/mock-account-server.sh
Samples: scripts/mock-account-server.sh --write-samples crates/account/src/samples
Environment: LAUNCH_FREE (open | until:<RFC 3339 UTC> | off), GRANTS_FILE (JSON path), TRIAL_SECONDS (default 7 days), PAID_EMAILS (comma-separated).
Google sign-in is faked by a form asking for an email. Rate limits (route 6) are not implemented.
"""

import argparse
import base64
import hashlib
import hmac
import html
import json
import os
import secrets
import sys
import threading
from datetime import datetime, timedelta, timezone
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, urlencode, urlparse

from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import ec, utils

HOST = "127.0.0.1"
PORT = 8787
KEY_ID = "test-1"
SAMPLE_DEVICE_HASH = "a" * 64
MAXIMUM_ACTIVE_DEVICES = 2
LOGIN_CODE_SECONDS = 120

# Test-only key: it signs nothing real, and the app's tests hold its public half.
PRIVATE_KEY_PEM = b"""-----BEGIN PRIVATE KEY-----
MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgA7lreD8b9awuTllc
zM4JbxGfOUjlzhJGbk6+WY4yE/ShRANCAATV1wBLbVRhiW/Ku0aBw2ZtHwE1J8MH
J6tjz7wfh8CABMRV5mlQB1IjnpziDvttnCfFNmJF5Hwer+NwoqpjhKLa
-----END PRIVATE KEY-----
"""
PRIVATE_KEY = serialization.load_pem_private_key(PRIVATE_KEY_PEM, password=None)


def base64url_encode(data: bytes) -> str:
    return base64.urlsafe_b64encode(data).rstrip(b"=").decode("ascii")


def sha256_hex(text: str) -> str:
    return hashlib.sha256(text.encode("ascii")).hexdigest()


def format_time(moment: datetime) -> str:
    return moment.astimezone(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def parse_time(text: str) -> datetime:
    return datetime.strptime(text, "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=timezone.utc)


def sign_payload(payload: dict, key=PRIVATE_KEY) -> str:
    payload_part = base64url_encode(json.dumps(payload, separators=(",", ":")).encode("utf-8"))
    der_signature = key.sign(payload_part.encode("ascii"), ec.ECDSA(hashes.SHA256()))
    r_value, s_value = utils.decode_dss_signature(der_signature)
    raw_signature = r_value.to_bytes(32, "big") + s_value.to_bytes(32, "big")
    return payload_part + "." + base64url_encode(raw_signature)


def public_key_hex() -> str:
    numbers = PRIVATE_KEY.public_key().public_numbers()
    return (numbers.x.to_bytes(32, "big") + numbers.y.to_bytes(32, "big")).hex()


def trial_seconds() -> int:
    return int(os.environ.get("TRIAL_SECONDS", str(7 * 24 * 3600)))


def launch_state(now: datetime) -> dict:
    setting = os.environ.get("LAUNCH_FREE", "off").strip()
    if setting == "open":
        return {"active": True}
    if setting.startswith("until:"):
        ends_at = parse_time(setting[len("until:"):])
        return {"active": now < ends_at, "ends_at": format_time(ends_at)}
    return {"active": False}


def build_policy(now: datetime) -> dict:
    policy = {"v": 1, "kind": "policy", "kid": KEY_ID, "launch": launch_state(now)}
    grants_path = os.environ.get("GRANTS_FILE")
    if grants_path:
        with open(grants_path, encoding="utf-8") as grants_file:
            policy["grants"] = json.load(grants_file)
    policy["issued_at"] = format_time(now)
    return policy


class Store:
    """In-memory users, trials, devices and login codes, guarded by one lock."""

    def __init__(self):
        self.lock = threading.Lock()
        self.users = {}  # email -> user_id
        self.trials = []  # {user_id, device_hash, started_at, ends_at}
        self.devices = []  # see add_device
        self.login_codes = {}  # code_hash -> {...}
        self.next_user_id = 1
        self.next_device_id = 1

    def user_id_for_email(self, email: str) -> str:
        if email not in self.users:
            self.users[email] = f"user-{self.next_user_id}"
            self.next_user_id += 1
        return self.users[email]

    def email_for_user(self, user_id: str) -> str:
        return next(email for email, known in self.users.items() if known == user_id)

    def active_devices(self, user_id: str) -> list:
        return [d for d in self.devices if d["user_id"] == user_id and d["signed_out_at"] is None]

    def earliest_active_device(self, user_id: str):
        return min(self.active_devices(user_id), key=lambda d: d["signed_in_at"])

    def needs_confirmation(self, user_id: str, device_hash: str) -> bool:
        active = self.active_devices(user_id)
        if any(d["device_hash"] == device_hash for d in active):
            return False
        return len(active) >= MAXIMUM_ACTIVE_DEVICES

    def apply_trial_start_rule(self, user_id: str, device_hash: str, now: datetime):
        if launch_state(now)["active"]:
            return
        if any(t["user_id"] == user_id or t["device_hash"] == device_hash for t in self.trials):
            return
        self.trials.append({
            "user_id": user_id,
            "device_hash": device_hash,
            "started_at": now,
            "ends_at": now + timedelta(seconds=trial_seconds()),
        })

    def plan_for(self, user_id: str, now: datetime):
        paid_emails = {e.strip().lower() for e in os.environ.get("PAID_EMAILS", "").split(",") if e.strip()}
        if self.email_for_user(user_id).lower() in paid_emails:
            return "paid", None
        for trial in self.trials:
            if trial["user_id"] == user_id and trial["ends_at"] > now:
                return "trial", trial["ends_at"]
        return "member", None

    def make_token(self, device: dict, now: datetime) -> str:
        plan, trial_ends_at = self.plan_for(device["user_id"], now)
        payload = {
            "v": 1,
            "kind": "device",
            "kid": KEY_ID,
            "user_id": device["user_id"],
            "device_id": device["device_id"],
            "device_hash": device["device_hash"],
            "plan": plan,
        }
        if trial_ends_at is not None:
            payload["trial_ends_at"] = format_time(trial_ends_at)
        payload["email"] = self.email_for_user(device["user_id"])
        payload["issued_at"] = format_time(now)
        return sign_payload(payload)

    def add_device(self, user_id: str, device_hash: str, name: str, now: datetime):
        secret = base64url_encode(secrets.token_bytes(32))
        secret_hash = hashlib.sha256(secret.encode("ascii")).hexdigest()
        existing = next((d for d in self.active_devices(user_id) if d["device_hash"] == device_hash), None)
        if existing:
            existing.update(name=name, secret_hash=secret_hash, signed_in_at=now)
            return existing, secret
        device = {
            "device_id": f"device-{self.next_device_id}",
            "user_id": user_id,
            "device_hash": device_hash,
            "name": name,
            "secret_hash": secret_hash,
            "signed_in_at": now,
            "last_check_at": None,
            "signed_out_at": None,
            "signed_out_reason": None,
        }
        self.next_device_id += 1
        self.devices.append(device)
        if len(self.active_devices(user_id)) > MAXIMUM_ACTIVE_DEVICES:
            earliest = self.earliest_active_device(user_id)
            earliest["signed_out_at"] = now
            earliest["signed_out_reason"] = "replaced"
        return device, secret

    def find_authenticated_device(self, device_id, device_secret):
        device = next((d for d in self.devices if d["device_id"] == device_id), None)
        if device is None or not isinstance(device_secret, str):
            return None
        presented_hash = hashlib.sha256(device_secret.encode("utf-8")).hexdigest()
        if not hmac.compare_digest(presented_hash, device["secret_hash"]):
            return None
        return device


STORE = Store()


def validate_login_query(query: dict):
    """Return (parameters, None) or (None, reason) for the query of /device/login."""
    values = {key: items[0] for key, items in query.items() if items}
    port_text = values.get("port", "")
    if not (port_text.isascii() and port_text.isdigit() and 1024 <= int(port_text) <= 65535):
        return None, "port"
    challenge = values.get("challenge", "")
    if len(challenge) != 43 or not all(c.isascii() and (c.isalnum() or c in "-_") for c in challenge):
        return None, "challenge"
    if not values.get("state"):
        return None, "state"
    device_hash = values.get("device_hash", "")
    if len(device_hash) != 64 or not all(c in "0123456789abcdefABCDEF" for c in device_hash):
        return None, "device_hash"
    name = values.get("name", "")
    if len(name) > 64:
        return None, "name"
    return {
        "port": port_text,
        "challenge": challenge,
        "state": values["state"],
        "device_hash": device_hash.lower(),
        "name": name,
    }, None


def callback_url(port: str, **query) -> str:
    return f"http://127.0.0.1:{int(port)}/callback?{urlencode(query)}"


def page(body: str) -> bytes:
    return (
        "<!doctype html><meta charset=utf-8><title>Notch mock account</title>"
        "<body style='font-family:sans-serif;max-width:32em;margin:3em auto'>" + body
    ).encode("utf-8")


def hidden_fields(parameters: dict, **extra) -> str:
    fields = {**parameters, **extra}
    return "".join(
        f"<input type=hidden name={html.escape(key)} value=\"{html.escape(str(value))}\">"
        for key, value in fields.items()
    )


class Handler(BaseHTTPRequestHandler):
    def log_message(self, format, *args):  # noqa: A002 - signature fixed by the base class
        sys.stderr.write("%s\n" % (format % args))

    def send_body(self, status: int, content_type: str, body: bytes, headers=None):
        self.send_response(status)
        self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(body)))
        for key, value in (headers or {}).items():
            self.send_header(key, value)
        self.end_headers()
        self.wfile.write(body)

    def send_json(self, status: int, payload: dict):
        self.send_body(status, "application/json", json.dumps(payload).encode("utf-8"))

    def send_redirect(self, location: str):
        self.send_body(302, "text/plain", b"", {"Location": location})

    def read_json(self):
        try:
            length = int(self.headers.get("Content-Length", "0"))
            body = json.loads(self.rfile.read(length) or b"{}")
            return body if isinstance(body, dict) else {}
        except (ValueError, UnicodeDecodeError):
            return {}

    def read_form(self) -> dict:
        length = int(self.headers.get("Content-Length", "0"))
        parsed = parse_qs(self.rfile.read(length).decode("utf-8"))
        return {key: items[0] for key, items in parsed.items() if items}

    def do_GET(self):
        url = urlparse(self.path)
        if url.path == "/api/policy":
            policy = build_policy(datetime.now(timezone.utc))
            self.send_body(
                200, "application/json", json.dumps({"policy": sign_payload(policy)}).encode("utf-8"),
                {"Cache-Control": "max-age=300"},
            )
        elif url.path == "/device/login":
            parameters, bad_field = validate_login_query(parse_qs(url.query))
            if bad_field:
                self.send_json(400, {"error": "invalid_request"})
                return
            form = (
                "<h1>Sign in with Google (mock)</h1><form method=post action=/device/login>"
                "<p>Google account email:<br><input name=email type=email required autofocus size=36></p>"
                + hidden_fields(parameters) + "<button>Continue with Google</button></form>"
            )
            self.send_body(200, "text/html; charset=utf-8", page(form))
        else:
            self.send_json(404, {"error": "not_found"})

    def do_POST(self):
        path = urlparse(self.path).path
        if path == "/device/login":
            self.handle_login_form()
        elif path == "/api/device/claim":
            self.handle_claim()
        elif path == "/api/device/check":
            self.handle_check()
        elif path == "/api/device/sign-out":
            self.handle_sign_out()
        else:
            self.send_json(404, {"error": "not_found"})

    def handle_login_form(self):
        form = self.read_form()
        parameters, bad_field = validate_login_query({key: [value] for key, value in form.items()})
        email = form.get("email", "").strip()
        if bad_field or "@" not in email:
            self.send_json(400, {"error": "invalid_request"})
            return
        with STORE.lock:
            user_id = STORE.user_id_for_email(email)
            if form.get("confirmed") != "yes" and STORE.needs_confirmation(user_id, parameters["device_hash"]):
                first = STORE.earliest_active_device(user_id)
                cancel_url = callback_url(parameters["port"], error="cancelled", state=parameters["state"])
                body = (
                    "<h1>This account is signed in on 2 PCs.</h1>"
                    f"<p>Continue signing in on {html.escape(parameters['name'])}? "
                    f"{html.escape(first['name'])}, signed in on {first['signed_in_at'].strftime('%Y-%m-%d')}, "
                    "will be signed out.</p>"
                    "<form method=post action=/device/login>"
                    + hidden_fields(parameters, email=email, confirmed="yes")
                    + f"<button>Continue</button> <a href=\"{html.escape(cancel_url)}\">Cancel</a></form>"
                )
                self.send_body(200, "text/html; charset=utf-8", page(body))
                return
            code = base64url_encode(secrets.token_bytes(32))
            STORE.login_codes[sha256_hex(code)] = {
                "user_id": user_id,
                "challenge": parameters["challenge"],
                "device_hash": parameters["device_hash"],
                "name": parameters["name"],
                "expires_at": datetime.now(timezone.utc) + timedelta(seconds=LOGIN_CODE_SECONDS),
                "used": False,
            }
        target = callback_url(parameters["port"], code=code, state=parameters["state"])
        body = (
            f"<meta http-equiv=refresh content=\"0;url={html.escape(target)}\">"
            f"<p>Signed in as {html.escape(email)}. Returning to Notch&hellip;</p>"
        )
        self.send_body(200, "text/html; charset=utf-8", page(body), {"Location": target})

    def handle_claim(self):
        request = self.read_json()
        code, verifier = request.get("code"), request.get("verifier")
        now = datetime.now(timezone.utc)
        if not isinstance(code, str) or not isinstance(verifier, str):
            self.send_json(400, {"error": "invalid_code"})
            return
        with STORE.lock:
            record = STORE.login_codes.get(sha256_hex(code)) if code.isascii() else None
            challenge = base64url_encode(hashlib.sha256(verifier.encode("utf-8")).digest())
            valid = (
                record is not None
                and not record["used"]
                and record["expires_at"] > now
                and hmac.compare_digest(challenge, record["challenge"])
            )
            if not valid:
                self.send_json(400, {"error": "invalid_code"})
                return
            record["used"] = True
            device, secret = STORE.add_device(record["user_id"], record["device_hash"], record["name"], now)
            STORE.apply_trial_start_rule(record["user_id"], record["device_hash"], now)
            token = STORE.make_token(device, now)
            device_id = device["device_id"]
        self.send_json(200, {
            "device_id": device_id,
            "device_secret": secret,
            "token": token,
            "policy": sign_payload(build_policy(now)),
        })

    def handle_check(self):
        request = self.read_json()
        now = datetime.now(timezone.utc)
        with STORE.lock:
            device = STORE.find_authenticated_device(request.get("device_id"), request.get("device_secret"))
            if device is None:
                self.send_json(401, {"error": "unknown_device"})
                return
            if device["signed_out_at"] is not None:
                self.send_json(410, {"error": "signed_out", "reason": device["signed_out_reason"]})
                return
            STORE.apply_trial_start_rule(device["user_id"], device["device_hash"], now)
            device["last_check_at"] = now
            token = STORE.make_token(device, now)
        self.send_json(200, {"token": token, "policy": sign_payload(build_policy(now))})

    def handle_sign_out(self):
        request = self.read_json()
        with STORE.lock:
            device = STORE.find_authenticated_device(request.get("device_id"), request.get("device_secret"))
            if device is not None and device["signed_out_at"] is None:
                device["signed_out_at"] = datetime.now(timezone.utc)
                device["signed_out_reason"] = "by_user"
        self.send_body(204, "text/plain", b"")


def write_samples(directory: str):
    os.makedirs(directory, exist_ok=True)
    issued_at = "2026-11-13T00:00:00Z"

    def token_payload(**overrides) -> dict:
        payload = {
            "v": 1, "kind": "device", "kid": KEY_ID, "user_id": "user-1", "device_id": "device-1",
            "device_hash": SAMPLE_DEVICE_HASH, "plan": "paid", "email": "sample@example.com",
            "issued_at": issued_at,
        }
        payload.update(overrides)
        return payload

    def policy_payload(launch: dict, grants=None) -> dict:
        payload = {"v": 1, "kind": "policy", "kid": KEY_ID, "launch": launch}
        if grants is not None:
            payload["grants"] = grants
        payload["issued_at"] = issued_at
        return payload

    valid_token = sign_payload(token_payload())
    _payload_part, signature_part = valid_token.split(".")
    tampered_payload = base64url_encode(json.dumps(token_payload(plan="member"), separators=(",", ":")).encode())
    grants = {
        "guest": {"media": ["page"], "calendar": ["page"], "shelf": ["page"], "activity": ["volume"]},
        "member": {"media": ["page", "lyrics"], "timer": []},
    }
    samples = {
        "token-valid.txt": valid_token,
        "token-tampered.txt": tampered_payload + "." + signature_part,
        "token-wrong-kid.txt": sign_payload(token_payload(kid="other-key")),
        "token-wrong-kind.txt": sign_payload(token_payload(kind="policy")),
        "token-trial-expired.txt": sign_payload(token_payload(plan="trial", trial_ends_at="2026-01-08T00:00:00Z")),
        "policy-launch-open.txt": sign_payload(policy_payload({"active": True})),
        "policy-launch-ended.txt": sign_payload(policy_payload({"active": False, "ends_at": "2026-01-01T00:00:00Z"})),
        "policy-with-grants.txt": sign_payload(policy_payload({"active": False}, grants)),
    }
    for file_name, content in samples.items():
        with open(os.path.join(directory, file_name), "w", encoding="ascii", newline="\n") as sample_file:
            sample_file.write(content + "\n")
    with open(os.path.join(directory, "test-1.public.hex"), "w", encoding="ascii", newline="\n") as key_file:
        key_file.write(public_key_hex() + "\n")
    print(f"wrote {len(samples) + 1} files to {directory}")


def main():
    parser = argparse.ArgumentParser(description="Mock Notch account server")
    parser.add_argument("--write-samples", metavar="DIRECTORY", help="write the test public key and signed samples, then exit")
    arguments = parser.parse_args()
    if arguments.write_samples:
        write_samples(arguments.write_samples)
        return
    server = ThreadingHTTPServer((HOST, PORT), Handler)
    print(f"mock account server on http://{HOST}:{PORT} (LAUNCH_FREE={os.environ.get('LAUNCH_FREE', 'off')})")
    server.serve_forever()


if __name__ == "__main__":
    main()
