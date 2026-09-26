#!/usr/bin/env python3
"""Build assets/library.json: every official karaoke upload from the labels' own channels.

Reads video metadata only (id, title, length) with `yt-dlp --flat-playlist`; nothing is downloaded.
Each title is parsed into song title / artist (and GMM's romanised title as a search alias). Longplays,
medleys and titles that do not parse are skipped, as are videos already in the curated
assets/catalog.json and duplicate uploads of the same song. Every video is checked for embedding with
YouTube oEmbed (same rule as tools/link_check.py), so the app never lists a song it cannot play.

Keypad codes are stable: a video keeps the code it got in the previous assets/library.json, and new
videos take the next free code in their channel's range.

Usage: python3 tools/harvest_library.py [--skip-embed-check]
"""
import json
import os
import re
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from link_check import status  # noqa: E402  (oEmbed: 200 = public + embeddable)

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
CATALOG = os.path.join(ROOT, "assets", "catalog.json")
LIBRARY = os.path.join(ROOT, "assets", "library.json")

MIN_SECS, MAX_SECS = 60, 600  # shorter: teasers/Shorts; longer: longplays and medleys
# "มีเสียงร้อง" = with the singer's vocals, i.e. not a karaoke track
SKIP_WORDS = re.compile(r"longplay|best hits|song book|medley|non-?stop|รวมเพลง|มีเสียงร้อง", re.I)
TAG = re.compile(r"\s*\[[^\]]*\]\s*")
ROMANISED = re.compile(r"\s*\(([A-Za-z0-9' .,&!?-]*-[A-Za-z0-9' .,&!?-]*)\)\s*")


def parse_gmm(title):
    """`คาราโอเกะ ชื่อเพลง (Rom-an-ised) - ศิลปิน [ Original Karaoke ]` -> (title, artist, alias)."""
    text = re.sub(r"^คาราโอเ+กะ\s*", "", title).replace(" – ", " - ")
    text = re.sub(r"\s*Uploaded$", "", TAG.sub(" ", text)).strip()
    match = ROMANISED.search(text)
    alias = ""
    if match:
        alias = re.sub(r"\s+", " ", match.group(1).replace("-", " ")).strip()
        head, tail = text[: match.start()], text[match.end():]
        # "ชื่อ (Rom-an) - ศิลปิน", or the dash was dropped: "ชื่อ (Rom-an)ศิลปิน"
        text = f"{head} - {tail[2:] if tail.startswith('- ') else tail.lstrip('- ')}"
    return split_dash(text) + (alias,) if " - " in text else None


def parse_title_first(title):
    """MuzikMove: `ชื่อเพลง (Alt) - ศิลปิน [คาราโอเกะ]`."""
    text = TAG.sub(" ", title).strip()
    return split_dash(text) + ("",) if " - " in text else None


def parse_artist_first(title):
    """Whattheduck: `ศิลปิน - ชื่อเพลง (Alt) [Official Karaoke]`."""
    text = TAG.sub(" ", title).strip()
    if " - " not in text:
        return None
    artist, song = split_dash(text)
    return song, artist, ""


def split_dash(text):
    left, right = text.split(" - ", 1)
    return left.strip(), right.strip()


# code ranges sit between the curated catalog's (100xx, 200xx) and the Add-URL range (90001+)
CHANNELS = [
    {"name": "GMM Karaoke", "url": "https://www.youtube.com/channel/UCHmKRqvKPYVx23RJ8uF6AtA/videos",
     "codes": (30001, 49999), "intro_skip_secs": 18, "parse": parse_gmm},
    {"name": "Whattheduck", "url": "https://www.youtube.com/playlist?list=PLGPdaIE_shQBC9qxgROu__RvAPIBIP0LN",
     "codes": (50001, 54999), "intro_skip_secs": 0, "parse": parse_artist_first},
    {"name": "Muzik Move Karaoke", "url": "https://www.youtube.com/@MuzikMoveKaraoke/videos",
     "codes": (55001, 59999), "intro_skip_secs": 0, "parse": parse_title_first},
]


def list_videos(url):
    out = subprocess.run(["yt-dlp", "--flat-playlist", "--print", "%(id)s\t%(duration)s\t%(title)s", url],
                         capture_output=True, text=True, check=True).stdout
    rows = [line.split("\t", 2) for line in out.splitlines() if line.count("\t") >= 2]
    return [(vid, int(float(secs)), title) for vid, secs, title in rows if secs not in ("NA", "None")]


def song_key(title, artist):
    return re.sub(r"\W+", "", f"{title}|{artist}".lower())


def harvest(channel, curated_ids, old_codes, check_embed):
    songs, seen, skipped = [], set(), 0
    for vid, secs, raw in list_videos(channel["url"]):
        parsed = None if SKIP_WORDS.search(raw) or not MIN_SECS <= secs <= MAX_SECS else channel["parse"](raw)
        if vid in curated_ids or parsed is None or not parsed[0] or not parsed[1]:
            skipped += 1
            continue
        key = song_key(parsed[0], parsed[1])
        if key in seen:  # newest upload of a song wins (channels list newest first)
            skipped += 1
            continue
        seen.add(key)
        songs.append({"vid": vid, "secs": secs, "title": parsed[0], "artist": parsed[1], "alias": parsed[2]})
    if check_embed:
        with ThreadPoolExecutor(max_workers=16) as pool:
            codes = list(pool.map(lambda s: status(s["vid"]), songs))
        dropped = sum(1 for c in codes if c is not None and c != 200)
        songs = [s for s, c in zip(songs, codes) if c is None or c == 200]  # network trouble: keep
        print(f"  {dropped} not embeddable", file=sys.stderr)
    assign_codes(songs, channel["codes"], old_codes)
    songs.sort(key=lambda s: s["code"])
    print(f"{channel['name']}: {len(songs)} songs ({skipped} skipped)", file=sys.stderr)
    return {"name": channel["name"], "intro_skip_secs": channel["intro_skip_secs"],
            "songs": [[s["vid"], s["code"], s["secs"], s["title"], s["artist"], s["alias"]] for s in songs]}


def assign_codes(songs, code_range, old_codes):
    first, last = code_range
    taken = set()
    for s in songs:
        code = old_codes.get(s["vid"])
        s["code"] = code if code is not None and first <= code <= last else None
        taken.add(s["code"])
    free = (c for c in range(first, last + 1) if c not in taken)
    for s in (s for s in songs if s["code"] is None):
        s["code"] = next(free)  # StopIteration = range full: widen it above


def to_json(library):
    """One song per line, so a re-harvest diffs as added / removed / changed songs."""
    compact = lambda value: json.dumps(value, ensure_ascii=False, separators=(",", ":"))  # noqa: E731
    channels = []
    for ch in library["channels"]:
        rows = ",\n".join(f"    {compact(row)}" for row in ch["songs"])
        head = f'  {{"name":{compact(ch["name"])},"intro_skip_secs":{ch["intro_skip_secs"]},"songs":[\n'
        channels.append(f"{head}{rows}\n  ]}}")
    return '{"channels":[\n' + ",\n".join(channels) + "\n]}\n"


def main():
    with open(CATALOG, encoding="utf-8") as f:
        curated_ids = {s["youtube_id"] for s in json.load(f)}
    old_codes = {}
    if os.path.exists(LIBRARY):
        with open(LIBRARY, encoding="utf-8") as f:
            old_codes = {row[0]: row[1] for ch in json.load(f)["channels"] for row in ch["songs"]}
    check_embed = "--skip-embed-check" not in sys.argv
    library = {"channels": [harvest(ch, curated_ids, old_codes, check_embed) for ch in CHANNELS]}
    with open(LIBRARY, "w", encoding="utf-8") as f:
        f.write(to_json(library))
    total = sum(len(ch["songs"]) for ch in library["channels"])
    print(f"wrote {total} songs to assets/library.json ({os.path.getsize(LIBRARY) // 1024} KB)", file=sys.stderr)


if __name__ == "__main__":
    main()
