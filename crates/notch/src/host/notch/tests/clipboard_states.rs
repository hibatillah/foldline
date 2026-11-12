//! Pictures of the clipboard page in each state of the design: cargo test clipboard_state -- --ignored.

use super::{images::save, *};

#[test]
#[ignore = "writes images to target/snapshots; run on purpose"]
fn clipboard_state_snapshots() {
    let scenarios = [
        ("state-clipboard-text", 0),
        ("state-clipboard-image", 1),
        ("state-clipboard-pasted", 2),
        ("state-clipboard-copied", 3),
        ("state-clipboard-removed", 4),
        ("state-clipboard-skipped", 5),
        ("state-clipboard-empty", 6),
        ("state-clipboard-pinned", 7),
        ("state-clipboard-pinned-full", 8),
    ];
    for (name, clipboard) in scenarios {
        let mut notch = rig(SampleOptions { clipboard, ..SampleOptions::default() });
        open(&mut notch, "clipboard");
        save(&notch, name, 260.0);
    }
}
