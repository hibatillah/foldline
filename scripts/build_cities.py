"""Builds assets/cities.tsv for the shalat page's city search from GeoNames cities15000 (CC BY 4.0).

Keeps name, country code, latitude, longitude and population, largest cities first, so a search lists the big ones first.
"""

import io
import urllib.request
import zipfile
from pathlib import Path

SOURCE = "https://download.geonames.org/export/dump/cities15000.zip"
OUTPUT = Path(__file__).resolve().parent.parent / "assets" / "cities.tsv"

NAME, LATITUDE, LONGITUDE, COUNTRY, POPULATION = 1, 4, 5, 8, 14


def main() -> None:
    with urllib.request.urlopen(SOURCE) as response:
        archive = zipfile.ZipFile(io.BytesIO(response.read()))
    rows = []
    with archive.open("cities15000.txt") as file:
        for line in io.TextIOWrapper(file, encoding="utf-8"):
            fields = line.rstrip("\n").split("\t")
            name = fields[NAME].replace("\t", " ").strip()
            if not name or not fields[COUNTRY]:
                continue
            latitude, longitude = round(float(fields[LATITUDE]), 3), round(float(fields[LONGITUDE]), 3)
            rows.append((int(fields[POPULATION] or 0), name, fields[COUNTRY], latitude, longitude))
    rows.sort(key=lambda row: (-row[0], row[1]))
    lines = [f"{name}\t{country}\t{latitude}\t{longitude}" for _, name, country, latitude, longitude in rows]
    OUTPUT.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"{len(lines)} cities, {OUTPUT.stat().st_size} bytes")


if __name__ == "__main__":
    main()
