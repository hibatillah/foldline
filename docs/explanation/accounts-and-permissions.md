# Accounts and permissions

> **Explanation.** Read when: you change who may use what, add a page or an activity and wonder why it needs a catalog row, or ask why a release build locks nothing yet. Skip if: you need the tables, statuses or file fields ([permissions reference](../reference/permissions.md)), the settings window's Account page ([settings window](../reference/settings-window.md#account)) or a way to try the sign-in ([mock account server](../how-to/run-the-mock-account-server.md)).

Notch is free to install. What an installed copy may use depends on one question, asked in one place: does this
account status allow this resource and action? This document gives the reasons for how that question is built. The
code is the crate `crates/account` (shared by `notch.exe` and `notch-settings.exe`, like `store`), the host's enforcement in
`crates/notch/src/host/notch/access.rs` and the daily check in `host/notch/daily_check.rs` and `host/account_check.rs`.

## The catalog is in code

`crates/account/src/catalog.rs` lists every resource and its actions. A resource is a page id (`media`, `weather`) or the
word `activity`, whose actions are the activity ids. The first action of a page is `page`, its base, and every other action
(`lyrics`, `rain_alert`) needs it: a page that is locked has none of its sub-features either.

The catalog is in code because the app can only enforce what it knows how to switch off. The site may change who gets a
feature; it cannot invent one the binary has no code for. Page and activity ids are the catalog's names, so the host asks
by id and stays ignorant of any particular page ([host and pages](host-and-pages.md)). The cost is one row per page or
activity; the test `every_registered_page_and_activity_is_in_the_catalog` fails until it exists.

## Grants are a table, and the site may override it per resource

Which status gets which action is data (`crates/account/src/grants.rs`), not an `if` in each page. One table can be read,
tested and changed in one place; the built-in values are in the [permissions reference](../reference/permissions.md#built-in-grants).

The signed policy the site sends may carry its own table. For each resource it names, the server's list replaces the
built-in list; a resource it does not name keeps the built-in one. The reasons for replacing per resource:

- An app older than the server's table does not lose a new feature the table has never heard of: unnamed resources keep
  what the app shipped with.
- The owner can move one feature between free and premium (or remove it from a status with `[]`) without a release and
  without restating the rest.
- Unknown resources and actions are ignored, so a table written for a newer app cannot break an older one.

A server table is refused as a whole (the built-in one stays; the policy's launch part still counts) when it would leave
the app unusable or inconsistent: a page action without `page`, or no page left for guests. A wrong table from the site
therefore cannot lock every guest out of the app. A table that names a status the app does not know (including `launch`,
which reads the `paid` grants) is dropped the same way.

## One enforcement point: the host

Only the host decides. It re-evaluates access at start-up, on resume, after the settings window changes a file, after the
daily check stores something, and when the notch opens, closes or is unpinned once a trial or launch end it knows about
has passed (a time comparison, no timer). From the result it:

- marks a page locked, which is a third state beside the user's on and off: the user's choices stay and return when
  access does, and a locked page is told it is off, so it stops its work and requests;
- does not start a locked activity, and hides the presence of one already started;
- keeps Windows' own volume flyout hidden only while both `volume` and `brightness` are allowed, and the notification
  banners only while `notifications` is allowed. When a lock removes the permission, the hidden popup is restored at once,
  so the user is never left with neither the notch's version nor Windows'.

A page asks for its own sub-features with `Context::allows(action)` and asks again in `on_access_changed`; it never sees
the status. With one place that decides, a new status or a new rule changes nothing in the pages, and no page can forget
to check.

## The settings window's locks are display only

The settings window shows a lock and **Sign in** or **Upgrade** on what the account does not unlock. It reads the same
catalog, grants and stored account as the notch, so the two agree, but it never gates anything: the switch it writes goes
to `settings.txt` as before, and the notch decides what that switch can do. If the window enforced, a second place could
disagree with the host, and a user who edits the file by hand would meet a different rule than one who uses the window.

## The daily check fails open

The notch asks the site at start-up and after resume from sleep, when 24 hours have passed since the last answer. It
sends one request on a thread of its own and wakes the UI thread through a `Notifier`; there is no timer and no retry
loop. A signed-in PC sends its device id and secret and gets a renewed token and policy; a signed-out one fetches the
policy alone and sends no identifier.

Only an answer from the site changes anything. A timeout, an error, a 5xx or a reply whose signatures do not verify leaves
the stored account as it was and leaves the check time alone, so the next start or resume tries again. The reasons:

- A user on a plane, behind a firewall or during a site outage keeps what they have. Locking a paying customer for a
  network failure costs more than letting a pirate keep access until the next successful check.
- The notch holds no state that expires on its own except what is signed into the token: a trial's end is a time stored in
  the token and compared with the clock, so a trial still ends while offline. What needs the network is revocation: a PC
  that the account signed out from another PC learns it at its next successful check (410, with the message "Signed out:
  this account signed in on another PC"), and a device the site no longer knows (401) signs out without a message.
- The server decides who holds a place among the account's two PCs. The app cannot, so it applies the answer when it
  arrives.

A tampered, copied or other-PC token is not trusted either way: it must verify against a key the app knows, as a token
of kind `device` for this PC's device hash. The daily check never loosens that; it only declines to take access away
without an answer.

## Why a release build locks nothing yet

Tokens and policies are signed with ECDSA P-256 and verified with Windows CNG against a public key compiled into the app
(`crates/account/src/keys.rs`). The production key list is empty: the site's production key does not exist yet. Without a
key, no token or policy can ever verify, so every signed-in user would read as a guest and a release would lock people
out for a reason they cannot fix.

`keys::is_enforced` is therefore true in debug builds (which know a test key, `test-1`) and in a release build only once
`PRODUCTION_KEYS` has an entry. While it is false, the stored account answers with the `launch` status and the built-in
grants, so everything is open and no locks show in the settings window. Adding the production key, and nothing else, turns
enforcement on in release builds.

## The free launch

The owner can run a free launch from the site (a policy field with an optional end time; no app release is needed to start
or end it). It gives the `paid` grants to every account that is **signed in**; a guest keeps the guest grants, because
the launch rule is read only after a valid token exists (the owner's decision: the launch is for accounts).

Status is the first match of these rules: no valid token is a guest; a paid plan is paid; a trial that has not ended is
a trial; a running launch is launch; anything else is a member. Paid comes before the launch so that an account that
paid keeps its status when the launch ends, and an ended trial falls through to a running launch instead of stopping at
member.

## Trials

The app reads a trial; the site grants it. A token carries the plan (`member`, `trial` or `paid`) and, for a trial, its
end time; a trial without an end time never counts as running. The rules for who gets one are the site's, and the plan is:
a trial is 7 days and goes to every account once and to every PC (by device hash) once, starting at sign-in or, for an
account made during the free launch, at its first check after the launch ends. The [mock account server](../how-to/run-the-mock-account-server.md)
implements the same rule for testing (`TRIAL_SECONDS` shortens it). The device hash binds a token to one PC so a copied
`account.txt` does not carry a trial to another.
