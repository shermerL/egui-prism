#!/usr/bin/env python3
"""Check that the built gallery is self-contained under its Project Pages path."""

from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import urlsplit

DIST = Path(__file__).resolve().parents[1] / "dist" / "egui-prism"
PREFIX = "/egui-prism/"


class Assets(HTMLParser):
    def __init__(self):
        super().__init__()
        self.urls = []

    def handle_starttag(self, tag, attrs):
        if tag in ("link", "script"):
            self.urls.extend(value for key, value in attrs if key in ("src", "href"))


def main():
    html = (DIST / "index.html").read_text()
    assets = Assets()
    assets.feed(html)
    if "data-trunk" in html:
        raise ValueError("Trunk directives remain in the built HTML")
    if not assets.urls:
        raise ValueError("The gallery has no generated asset links")
    for url in assets.urls:
        parsed = urlsplit(url)
        if parsed.scheme or parsed.netloc or not parsed.path.startswith(PREFIX):
            raise ValueError(f"Asset is outside the Project Pages path: {url}")
        path = DIST / parsed.path.removeprefix(PREFIX)
        if not path.resolve().is_relative_to(DIST.resolve()) or not path.is_file():
            raise ValueError(f"Missing or invalid asset: {url}")
    wasm = list(DIST.glob("*.wasm"))
    if len(wasm) != 1 or wasm[0].read_bytes()[:8] != b"\0asm\x01\0\0\0":
        raise ValueError("Expected one valid WebAssembly module")
    if not list(DIST.glob("*.js")):
        raise ValueError("Missing generated JavaScript")
    for name in ("LICENSE", "THIRD_PARTY_NOTICES.md"):
        if not (DIST / name).is_file():
            raise ValueError(f"Missing license notice: {name}")
    print(f"Project Pages assets verified: {len(assets.urls)} links, {wasm[0].stat().st_size} wasm bytes")


if __name__ == "__main__":
    main()
