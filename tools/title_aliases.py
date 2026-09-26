#!/usr/bin/env python3
"""Search aliases from the official karaoke video titles, so "rak mai wai" finds "รักไม่ไหวแล้วโว้ย".

GMM Karaoke titles carry the channel's own romanisation in parentheses, e.g.
"คาราโอเกะ รักไม่ไหวแล้วโว้ย (Rak-Mai-Wai-Laew-Voi) - ...". Only Latin-letter parentheses that are not
already part of the song title become an alias ("Rak Mai Wai Laew Voi"). Titles come from YouTube oEmbed
(no API key, no media download). Stdlib only.

Usage: python3 tools/title_aliases.py           (print proposed aliases)
       python3 tools/title_aliases.py --write   (store them in assets/catalog.json, keeping its format)
"""
import json
import os
import re
import sys
import urllib.parse
import urllib.request
from concurrent.futures import ThreadPoolExecutor

CATALOG = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "assets", "catalog.json")
OEMBED = "https://www.youtube.com/oembed?format=json&url="
ROMANISED = re.compile(r"\(([A-Za-z][A-Za-z' -]*[A-Za-z])\)")


def video_title(video_id):
    url = OEMBED + urllib.parse.quote(f"https://www.youtube.com/watch?v={video_id}", safe="")
    req = urllib.request.Request(url, headers={"User-Agent": "ktv-rs-title-aliases"})
    with urllib.request.urlopen(req, timeout=15) as resp:
        return json.load(resp)["title"]


def aliases_for(song, title):
    found = []
    for match in ROMANISED.findall(title):
        alias = " ".join(match.replace("-", " ").split())
        letters = lambda text: re.sub(r"[^a-z]", "", text.lower())
        if letters(alias) not in letters(song["title"]) and alias not in found:
            found.append(alias)
    return found


def with_aliases(song, aliases):
    """Same key order as `Song` in src/types.rs: aliases right after artist, omitted when empty."""
    out = {}
    for key, value in song.items():
        if key == "aliases":
            continue
        out[key] = value
        if key == "artist" and aliases:
            out["aliases"] = aliases
    return out


def main():
    write = "--write" in sys.argv[1:]
    with open(CATALOG, encoding="utf-8") as f:
        catalog = json.load(f)
    with ThreadPoolExecutor(max_workers=8) as pool:
        titles = list(pool.map(lambda s: video_title(s["youtube_id"]), catalog))
    updated = []
    for song, title in zip(catalog, titles):
        aliases = aliases_for(song, title)
        print(f'{song["code"]}  {song["title"]}  ->  {aliases or "-"}')
        updated.append(with_aliases(song, aliases))
    if write:
        tmp = CATALOG + ".tmp"
        with open(tmp, "w", encoding="utf-8") as f:
            f.write(json.dumps(updated, indent=2, ensure_ascii=False) + "\n")
        os.replace(tmp, CATALOG)
        print(f"wrote {sum(1 for s in updated if 'aliases' in s)} songs with aliases")


if __name__ == "__main__":
    main()
