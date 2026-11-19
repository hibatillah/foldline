# Contributing to Foldline

Foldline is open source under the GNU General Public License, version 3 or later ([LICENSE](LICENSE)), and takes contributions through pull requests.

- Setup, structure and the rules the code follows: [ONBOARDING.md](ONBOARDING.md).
- Work on a branch named `type/short-name` and open a pull request into `master`. A pull request merges when its **Check** job is green.
- A change is ready when `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass, run as `cargo +stable-x86_64-pc-windows-msvc` (add `--workspace` when `crates/settings/`, `crates/store/` or `crates/places/` changed).
- The name and logo are not part of the code license: [TRADEMARK.md](TRADEMARK.md).

## Pull requests

- Title: `type(scope): summary`. Description: short bullets on what changed and why.
- Add tests for the logic you change, and update the docs that cover anything a user can see.
- Submit only what you can give under the agreement below. That excludes code from a project with an incompatible license and files of someone else's work.

## Contributor agreement

Opening a pull request, or pushing a commit to one, means you accept this agreement. The pull request template repeats it with a box to tick, and the maintainer may ask you to confirm it in writing before merging. The **maintainer** is Hibatillah, the owner of this repository. A **contribution** is any code, text, image or other material you submit to it.

1. **Your right to contribute.** The contribution is your own work, or you have the right to submit it under these terms. It contains no material under a license that the GNU GPL version 3 or later does not allow.
2. **License.** You license the contribution under the GNU General Public License, version 3 or later, like the rest of the project. You keep the copyright.
3. **Distribution by the maintainer.** You give the maintainer a worldwide, perpetual, irrevocable, royalty-free, non-exclusive license to copy, modify and distribute the contribution, in source and binary form, as part of Foldline, through any channel. This includes the Microsoft Store, free of charge. The maintainer may sublicense the contribution to Microsoft only as far as the Store needs to distribute Foldline for the maintainer. None of this takes away a right the GNU GPL gives to everyone who receives the source.
4. **Free of charge.** The maintainer distributes Foldline free of charge, in the Microsoft Store and everywhere else, and takes no profit from your contribution: no price, no paid tiers, no in-app purchases and no advertising.
5. **No warranty.** The contribution is provided as it is, as the GNU GPL describes.

If you can't agree to this, open an issue instead of a pull request.
