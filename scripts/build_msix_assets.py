"""Writes the MSIX logo PNGs in installer/msix/Assets from the blue tile (assets/branding/tile-stable.png, 256 px).

The tile is only ever scaled down, so every size whose tile would exceed 256 px is left out.
"""

from pathlib import Path

from PIL import Image

REPOSITORY = Path(__file__).resolve().parent.parent
SOURCE = REPOSITORY / "assets" / "branding" / "tile-stable.png"
OUTPUT = REPOSITORY / "installer" / "msix" / "Assets"

# Taskbar, Start list and title bar icons: the tile fills the whole square.
SQUARE_44_SCALES = {100: 44, 200: 88, 400: 176}
SQUARE_44_TARGET_SIZES = [16, 24, 32, 48, 256]
STORE_LOGO_SCALES = {100: 50, 200: 100, 400: 200}
# Start tiles: the tile takes two thirds of the height, centred on a transparent canvas.
SQUARE_150_SCALES = {100: 150, 150: 225, 200: 300}
WIDE_310_SCALES = {100: (310, 150), 200: (620, 300)}
TILE_SHARE_OF_HEIGHT = 2 / 3


def scaled_tile(tile: Image.Image, side: int) -> Image.Image:
    """Resized with premultiplied alpha, so the transparent corners do not darken the edge."""
    return tile.convert("RGBa").resize((side, side), Image.Resampling.LANCZOS).convert("RGBA")


def centred(tile: Image.Image, width: int, height: int) -> Image.Image:
    side = round(height * TILE_SHARE_OF_HEIGHT)
    canvas = Image.new("RGBA", (width, height), (0, 0, 0, 0))
    canvas.alpha_composite(scaled_tile(tile, side), ((width - side) // 2, (height - side) // 2))
    return canvas


def save(image: Image.Image, name: str) -> None:
    image.save(OUTPUT / name, optimize=True)
    print(f"{name}: {image.width}x{image.height}")


def main() -> None:
    tile = Image.open(SOURCE).convert("RGBA")
    if tile.size != (256, 256):
        raise SystemExit(f"{SOURCE} is {tile.size}, expected 256x256")
    OUTPUT.mkdir(parents=True, exist_ok=True)
    for old in OUTPUT.glob("*.png"):
        old.unlink()

    for scale, side in SQUARE_44_SCALES.items():
        save(scaled_tile(tile, side), f"Square44x44Logo.scale-{scale}.png")
    for side in SQUARE_44_TARGET_SIZES:
        image = scaled_tile(tile, side)
        save(image, f"Square44x44Logo.targetsize-{side}.png")
        # The tile carries its own background, so the unplated (taskbar) form is the same picture.
        save(image, f"Square44x44Logo.altform-unplated_targetsize-{side}.png")
    for scale, side in STORE_LOGO_SCALES.items():
        save(scaled_tile(tile, side), f"StoreLogo.scale-{scale}.png")
    for scale, side in SQUARE_150_SCALES.items():
        save(centred(tile, side, side), f"Square150x150Logo.scale-{scale}.png")
    for scale, (width, height) in WIDE_310_SCALES.items():
        save(centred(tile, width, height), f"Wide310x150Logo.scale-{scale}.png")


if __name__ == "__main__":
    main()
