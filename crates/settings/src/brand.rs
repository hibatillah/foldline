//! The app's tile logo: the blue tile of a stable build, the orange tile of a pre-release build.

use std::sync::{Arc, OnceLock};

use gpui::{Image, ImageFormat};

use crate::{release_version, version};

const TILE_STABLE: &[u8] = include_bytes!("../../../assets/branding/tile-stable.png");
const TILE_DEV: &[u8] = include_bytes!("../../../assets/branding/tile-dev.png");

/// The tile of the build's channel: the dev tile for a pre-release version, the stable tile otherwise (as the executables' icons).
fn tile_bytes(version: &str) -> &'static [u8] {
    if release_version::icon_for_version(Some(version)) == release_version::DEV_ICON { TILE_DEV } else { TILE_STABLE }
}

/// The running build's tile, decoded once.
pub fn tile_image() -> Arc<Image> {
    static TILE: OnceLock<Arc<Image>> = OnceLock::new();
    TILE.get_or_init(|| {
        Arc::new(Image::from_bytes(ImageFormat::Png, tile_bytes(version::displayed_version()).to_vec()))
    })
    .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stable_version_gets_the_stable_tile_and_a_pre_release_the_dev_tile() {
        assert_eq!(tile_bytes("0.5.0"), TILE_STABLE);
        assert_eq!(tile_bytes("0.5.0-alpha.2"), TILE_DEV);
    }
}
