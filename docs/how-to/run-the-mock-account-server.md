# Run the mock account server

Read this to try the app against the account API before the real site exists; skip it once the site is live.

The site that sells Notch does not exist yet. `scripts/mock-account-server.py` stands in for it on `127.0.0.1:8787`:
the device API (login page, `claim`, `check`, `sign-out`, `policy`) with an in-memory store, so restarting it forgets
every account and PC. It does not limit request rates.

## Start it

```bash
scripts/mock-account-server.sh
```

| Variable | Meaning |
| --- | --- |
| `LAUNCH_FREE` | `open`, `until:2026-12-31T17:00:00Z` or `off` (default) |
| `GRANTS_FILE` | path to a grants JSON file; when unset the policy carries no grants |
| `TRIAL_SECONDS` | length of a trial in seconds (default 7 days); set a few seconds to test a trial ending |
| `PAID_EMAILS` | comma-separated emails whose accounts get plan `paid` in tokens |

Check it with `curl -s http://127.0.0.1:8787/api/policy`, which returns `{"policy": "<base64url>.<base64url>"}`.

## Point the app at it

A debug build of the notch and of the settings window uses `http://127.0.0.1:8787` unless `NOTCH_ACCOUNT_URL` names another
address, and verifies signatures with the test key. A release build, with no production key, locks nothing and
makes no account request at all (neither program signs in, checks or opens the site), so it cannot show the mock's
effects. To make a real PC's token verify against the signed samples, set
`NOTCH_DEVICE_HASH_OVERRIDE` to 64 hex characters (the samples use `"a"` repeated 64 times). To skip the server and force a
status, set `NOTCH_ACCOUNT_STATUS`. All three are in [permissions](../reference/permissions.md#debug-environment-variables).

## Sign in

`/device/login` asks for an email, which stands in for the Google account; any address works. With two PCs already
signed in on that email and a new `device_hash`, it asks **Continue** or **Cancel**, naming the PC that signed in first:
Continue signs that PC out (its next `check` answers 410 `replaced`), Cancel returns to the app with `error=cancelled`.
Otherwise it redirects to `http://127.0.0.1:<port>/callback?code=...&state=...`.

## Test key and samples

Tokens and policies are signed with a fixed test key (kid `test-1`) that is in the script and protects nothing real.
Regenerate the files the app's tests read:

```bash
scripts/mock-account-server.sh --write-samples crates/account/src/samples
```

This writes `test-1.public.hex` (raw X then Y, hex) and eight signed samples, all for the device hash `"a" * 64`.
