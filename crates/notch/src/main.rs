#![windows_subsystem = "windows"]

rust_i18n::i18n!("locales", fallback = "en");

mod activities;
mod activity;
mod gfx;
mod host;
mod i18n;
mod log;
mod page;
mod pages;
mod platform;
#[cfg(test)]
mod source_scan;
mod ui;

fn main() {
    if let Err(error) = host::run(pages::all(), activities::all()) {
        log::debug(&format!("fatal: {error}"));
    }
}
