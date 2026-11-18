"""Writes the installer's header images: the channel tile at the sizes Inno Setup picks by display scale, padded so it sits inside the header band."""
from pathlib import Path

from PIL import Image

root = Path(__file__).resolve().parent.parent
sizes = [55, 64, 72, 80, 89, 97, 106, 114, 123]
fill = 0.82

for tile in ("tile-stable", "tile-dev"):
    source = Image.open(root / "assets" / "branding" / f"{tile}.png").convert("RGBA")
    for size in sizes:
        canvas = Image.new("RGBA", (size, size), (0, 0, 0, 0))
        inner = round(size * fill)
        resized = source.resize((inner, inner), Image.LANCZOS)
        offset = (size - inner) // 2
        canvas.paste(resized, (offset, offset), resized)
        canvas.save(root / "installer" / "wizard" / f"{tile}-{size}.png")
