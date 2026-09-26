#!/usr/bin/env python3
"""Link-rot check: every karaoke and guide video in assets/catalog.json must still be embeddable.

Uses YouTube oEmbed (no API key): 200 = public + embeddable, 401 = embedding disabled,
400/403/404 = removed or private. Stdlib only, so CI needs no pip install.

Usage: python3 tools/link_check.py            (exit 1 and a Markdown report on stdout if any link is dead)
"""
import json
import os
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from concurrent.futures import ThreadPoolExecutor

CATALOG = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "assets", "catalog.json")
OEMBED = "https://www.youtube.com/oembed?format=json&url="
RETRIES = 3
REASONS = {401: "embedding disabled", 400: "removed or invalid", 403: "private", 404: "removed"}


def status(video_id):
    url = OEMBED + urllib.parse.quote(f"https://www.youtube.com/watch?v={video_id}", safe="")
    for attempt in range(RETRIES):
        try:
            with urllib.request.urlopen(urllib.request.Request(url, headers={"User-Agent": "ktv-rs-link-check"}),
                                        timeout=15) as resp:
                return resp.status
        except urllib.error.HTTPError as e:
            if e.code < 500 and e.code != 429:
                return e.code
        except (urllib.error.URLError, TimeoutError):
            pass
        time.sleep(2 ** attempt)
    return None  # network trouble, not link rot


def main():
    with open(CATALOG, encoding="utf-8") as f:
        catalog = json.load(f)
    links = [(song, "karaoke", song["youtube_id"]) for song in catalog]
    links += [(song, "guide", song["guide"]["video_id"]) for song in catalog if song.get("guide")]
    with ThreadPoolExecutor(max_workers=8) as pool:
        codes = list(pool.map(lambda link: status(link[2]), links))

    dead = [(link, code) for link, code in zip(links, codes) if code is not None and code != 200]
    unknown = [link for link, code in zip(links, codes) if code is None]
    print(f"Checked {len(links)} videos: {len(links) - len(dead) - len(unknown)} ok, {len(dead)} dead, "
          f"{len(unknown)} unreachable")
    if dead:
        print("\n| Code | Song | Kind | Video | Status |\n| --- | --- | --- | --- | --- |")
        for (song, kind, video_id), code in dead:
            print(f"| {song['code']} | {song['title']} — {song['artist']} | {kind} | "
                  f"[{video_id}](https://www.youtube.com/watch?v={video_id}) | {code} {REASONS.get(code, '')} |")
        print("\nFix: replace the video in `assets/catalog.json` (guide: re-measure with "
              "`tools/guide_align.py --write`, or remove `guide` so the song falls back to karaoke audio).")
    for song, kind, video_id in unknown:
        print(f"unreachable (network): {song['code']} {kind} {video_id}", file=sys.stderr)
    sys.exit(1 if dead else 0)


if __name__ == "__main__":
    main()
