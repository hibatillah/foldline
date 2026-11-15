# Permissions

> **Reference.** Read when: you need the catalog, the statuses, which status gets what, how a server table merges, the debug environment variables or the fields of `account.txt`. Skip if: you want the reasons ([accounts and permissions](../explanation/accounts-and-permissions.md)), the Account page of the settings window ([settings window](settings-window.md#account)) or to add a page or an activity ([add a page](../how-to/add-a-page.md), [add an activity](../how-to/add-an-activity.md)).

Source: `crates/account/src/` (`catalog.rs`, `grants.rs`, `status.rs`, `access.rs`, `keys.rs`, `stored.rs`, `schedule.rs`,
`payload.rs`), the host in `crates/notch/src/host/notch/access.rs`.

## Catalog

`catalog.rs`. A resource is a page id, or `activity`. For a resource with a base, the first action is `page` and every other
action needs it; `activity` has no base, each action is its own.

| Resource | Actions |
| --- | --- |
| `media` | `page`, `lyrics` |
| `calendar` | `page`, `google` |
| `shelf`, `timer`, `todo`, `shalat`, `stats`, `devices`, `clipboard` | `page` |
| `weather` | `page`, `rain_alert` |
| `activity` | `volume`, `brightness`, `battery`, `caps_lock`, `microphone`, `camera`, `bluetooth`, `device_battery`, `notifications`, `clipboard`, `downloads`, `account_notice` |

`activity` actions are the activity ids. `calendar`/`google` has no code behind it yet: no page asks for it. `account_notice`
is the message shown after the account signed in on another PC; it is granted to every status, so it is never locked.
A page or activity id the catalog does not name is never locked by the host, and the test
`every_registered_page_and_activity_is_in_the_catalog` (`host/notch/tests/access.rs`) fails until a registered id has its row.

## Statuses

`status.rs`. The first matching rule wins.

| Order | Status | When |
| --- | --- | --- |
| 1 | `guest` | no valid token on this PC |
| 2 | `paid` | the token's plan is `paid` |
| 3 | `trial` | the plan is `trial` and `trial_ends_at` is in the future (a trial without `trial_ends_at` does not count) |
| 4 | `launch` | the stored policy's `launch.active` is true and it has no `ends_at` or one in the future |
| 5 | `member` | anything else |

A token is valid when its signature verifies against a known key, it is of kind `device`, its `v` is 1, its `kid` is a
known key and its `device_hash` equals this PC's. `launch` reads the `paid` grants.

When `keys::is_enforced()` is false (a release build with no production key), the stored account answers `launch` with the
built-in grants whatever it holds; see [Enforcement of signatures](#enforcement-of-signatures).

## Built-in grants

`Grants::built_in()` in `grants.rs`.

| Resource | `guest`, `member` | `trial`, `paid`, `launch` |
| --- | --- | --- |
| `media` | `page` | `page`, `lyrics` |
| `calendar` | `page` | `page`, `google` |
| `shelf` | `page` | `page` |
| `timer`, `todo`, `shalat`, `stats`, `devices`, `clipboard` | none | `page` |
| `weather` | none | `page`, `rain_alert` |
| `activity` | `volume`, `brightness`, `battery`, `caps_lock`, `microphone`, `camera`, `downloads`, `account_notice` | all twelve actions |

Not granted to `guest` and `member`: the activities `bluetooth`, `device_battery`, `notifications` and `clipboard`.
`downloads` and `account_notice` are granted to every status. `guest` and `member` have the same grants.

## Grants from the server

The signed policy may carry `grants`: `{"<status>": {"<resource>": ["<action>", ...]}}`, with the statuses `guest`,
`member`, `trial` and `paid`.

- **Per resource, replace.** For each status, a resource named in the table replaces the built-in actions of that resource
  for that status. A resource it does not name keeps the built-in ones.
- **`[]` removes.** An empty list leaves that status no action of the resource.
- **Unknown names are ignored.** A resource not in the catalog is skipped; an action not in the catalog for that resource is
  dropped from its list.
- **Refused tables.** The whole table is dropped (the built-in grants stay; the policy's `launch` still applies) when:
  - a resource with a base names at least one known action but not `page` (`["rain_alert"]` is refused; `[]` and
    `["karaoke"]` are not);
  - after merging, no page at all is granted to `guest`;
  - a status key is not `guest`, `member`, `trial` or `paid` (a `launch` key included);
  - `grants` is not an object of objects of string lists (a malformed table reads as absent).
- A policy whose own fields are wrong (no `launch`, a bad time, a bad signature) is not read at all.

## What the host enforces

`host/notch/access.rs` re-evaluates at start-up, on resume, after `WM_SETTINGS_CHANGED`, after the daily check saves, and
when the notch opens, closes or is unpinned once a known trial or launch end has passed.

| Lock on | Effect |
| --- | --- |
| a page (`<page id>`/`page`) | the page is locked in the page list (the user's on and off stay) and gets `on_enabled_changed(false)` when that changes its shown state; when every page the user chose is locked, the unlocked pages that are not opt-in show instead; `on_access_changed` runs on every page after any change |
| a sub-feature | the page asks `Context::allows(action)` and `on_access_changed` again |
| an activity | not started while locked; one already started shows nothing |
| `activity`/`volume` or `activity`/`brightness` | Windows' volume flyout stays hidden only while both are allowed |
| `activity`/`notifications` | Windows' notification banners stay hidden only while it is allowed |

When a popup's permission is lost, it is restored at once, unless the notch is paused (the resume applies it).

## Settings window

`crates/settings/src/standing.rs` reads the same catalog, grants and stored account. A row shows a lock when the action is
in the catalog, the account does not allow it and the build enforces: **Sign in** for a guest, **Upgrade** for any other
status. Display only. Details in [settings window](settings-window.md#account).

## Enforcement of signatures

`keys.rs`. `PRODUCTION_KEYS` is empty today. `is_enforced()` is true in debug builds and in a release build with at least one
production key. The test key `test-1` is known in debug builds only. While not enforced, `StoredAccount::access` returns
status `launch` with the built-in grants: everything is allowed, signed in or not, and a server table is not applied.

## Debug environment variables

Read in debug builds only (`cfg!(debug_assertions)`); a release build ignores all three.

| Variable | Read by | Effect |
| --- | --- | --- |
| `NOTCH_ACCOUNT_STATUS` | `notch.exe` at start (`host/notch/access.rs`) and `notch-settings.exe` (`standing.rs`) | `guest`, `member`, `trial`, `paid` or `launch` (any case) forces the status; the grants stay built in and nothing is read from `account.txt` for the status. Another value is ignored. The settings window reads it the same way (`account::status::parse_forced_status`) for its status and lock badges |
| `NOTCH_ACCOUNT_URL` | both programs (`account::client::site_url`) | the site's address, default `http://127.0.0.1:8787`; a trailing `/` is removed and an empty value uses the default. Release builds use `https://fold.encure.id` |
| `NOTCH_DEVICE_HASH_OVERRIDE` | both programs (`account::device::device_hash`) | replaces the PC's device hash when it is exactly 64 hex characters (lowercased); otherwise ignored. The signed samples are for `"a"` repeated 64 times |

## `account.txt`

`%APPDATA%\notch\account.txt` (`stored.rs`), `key=value` one per line, written to a `.tmp` beside it and renamed over. Both
programs read it; the notch loads it again on `WM_SETTINGS_CHANGED` and on resume.

| Field | Content |
| --- | --- |
| `device_id` | the id the site gave this PC |
| `device_secret` | the secret, DPAPI-protected (`CryptProtectData`) and base64url-encoded; never written in plain text |
| `token` | `<base64url payload>.<base64url signature>`, kind `device`: `user_id`, `device_id`, `device_hash`, `email`, `plan` (`member`, `trial` or `paid`), optional `trial_ends_at`, `issued_at`, `kid`, `v` (the version, a whole number, 1) |
| `policy` | the same form, kind `policy`: `launch` (`active`, optional `ends_at`), optional `grants`, `issued_at`, `kid`, `v` |
| `last_check_at` | Unix seconds of the last check the site answered |

Times in a payload are `YYYY-MM-DDTHH:MM:SSZ` only. Reading rules:

- A missing, unreadable or corrupt file reads as signed out; an empty value, a line without `=` and an unknown key are skipped.
- `device_id` and `device_secret` stand or fall together: if one is missing, or the secret cannot be unprotected (another
  Windows user, a copied file), both and the token are cleared; the policy stays.
- A value with a line break is not saved.
- A secret or token is never logged (`Debug` redacts them).

## The daily check

`schedule.rs`, `host/notch/daily_check.rs`, `host/account_check.rs`, `account::client`.

- Due when `last_check_at` is missing, 86 400 seconds old or more, or in the future. Tried at start-up and after resume.
  One request at a time, on its own thread, at most 15 seconds (`REQUEST_TIMEOUT`).
- Signed in: `POST /api/device/check` with `device_id` and `device_secret`. Signed out: `GET /api/policy`, no identifier.

| Reply | Result |
| --- | --- |
| 200 with a token and policy that verify (guest: a policy) | token (when sent) and policy stored, `last_check_at` set, access re-evaluated |
| 200 whose token or policy does not verify | nothing changes, `last_check_at` kept |
| 410 | `device_id`, `device_secret` and `token` removed, policy kept, `last_check_at` set, pill message "Signed out: this account signed in on another PC" |
| 401 | the same without the message |
| error, timeout, other status | nothing changes, `last_check_at` kept; tried again at the next start or resume, never in a loop |

A result for a device other than the one now in `account.txt` (the settings window signed in or out meanwhile) is dropped.
