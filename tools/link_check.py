#!/usr/bin/env python3
"""Link-rot check: every video the app plays must still be embeddable: the curated catalog (karaoke + guide), the
full library's karaoke videos (assets/library.json) and the official videos in assets/mv_guides.json.

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

ASSETS = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "assets")
CATALOG = os.path.join(ASSETS, "catalog.json")
LIBRARY = os.path.join(ASSETS, "library.json")
MV_GUIDES = os.path.join(ASSETS, "mv_guides.json")
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


def load(path):
    with open(path, encoding="utf-8") as f:
        return json.load(f)


def all_links():
    """(song, kind, video id) for every video the app can play; `song` has code / title / artist."""
    catalog = load(CATALOG)
    links = [(song, "karaoke", song["youtube_id"]) for song in catalog]
    links += [(song, "guide", song["guide"]["video_id"]) for song in catalog if song.get("guide")]
    library = {}
    for channel in load(LIBRARY)["channels"]:
        for vid, code, _secs, title, artist, _alias in channel["songs"]:
            library[vid] = {"code": code, "title": title, "artist": artist, "channel": channel["name"]}
    links += [(song, "library", vid) for vid, song in library.items()]
    links += [(library[vid], "mv", guide["video_id"]) for vid, guide in load(MV_GUIDES).items() if vid in library]
    return links


def main():
    links = all_links()
    with ThreadPoolExecutor(max_workers=16) as pool:
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
        print("\nFix: **karaoke / guide** — replace the video in `assets/catalog.json` (guide: re-time it with the "
              "in-app Guide Timing Tools and paste its Copy JSON, or remove `guide` so the song falls back to karaoke "
              "audio). **library** — re-run `python3 tools/harvest_library.py` (it drops videos that no longer embed; "
              "their codes are retired). **mv** — re-run `python3 tools/match_mv.py` (a timed entry must be removed "
              "or re-timed by hand in `assets/mv_guides.json`).")
    for song, kind, video_id in unknown:
        print(f"unreachable (network): {song['code']} {kind} {video_id}", file=sys.stderr)
    sys.exit(1 if dead else 0)


if __name__ == "__main__":
    main()
