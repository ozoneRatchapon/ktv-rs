"""Auto-timing from the official audio track (metadata only; used by tools/match_mv.py).

A GMM Karaoke video is an 18 s channel intro followed by the studio recording. The label's official audio track on
YouTube Music is that same recording with nothing before it, so the guide lines up at a fixed offset. Checked against
the hand-timed catalog (2026-09-27): where the track matches the karaoke length (16-22 s shorter), 11 of 12 hand
timings lie within 0.3 s of -18.2 s and the 12th within 0.9 s; every track outside the length window was rejected.

Only the first YouTube Music song result for "title artist" is taken (a looser pick once chose another artist's
cover); it must be a music track (has `track` metadata), carry the song's Thai or romanised title, not be a
live / remix / cover / other version, and fit the length window. Nothing is played, recorded or downloaded.
"""
import json
import os
import re
import subprocess
import sys
import time
import urllib.parse
from concurrent.futures import CancelledError, as_completed

OFFSET_SECS = -18.2  # GMM Karaoke intro, measured on the hand-timed catalog
GAP = (16, 22)       # karaoke length minus track length that means "same recording after the intro"
# Karaoke channels whose videos are a fixed intro + the studio recording
CHANNELS = {"GMM Karaoke"}
OTHER_VERSION = re.compile(r"live|remix|cover|acoustic|อคูสติก|อะคูสติก|version|เวอร์ชั่น|instrumental|karaoke|"
                           r"backing|big band|sped|slowed|demo|ost\.? ver", re.I)
TRIES = 3            # attempts per yt-dlp call before the song counts as failed (not as "no track")
CHECKPOINT = 100     # songs looked up between cache saves, so an interrupted run resumes
MAX_FAILED_IN_A_ROW = 25  # YouTube is blocking or the network is down: stop instead of burning the queue


class LookupFailed(Exception):
    """yt-dlp failed (rate limit, network): the song is unknown, not trackless, and is retried on the next run."""


def run(args):
    for attempt in range(TRIES):
        done = subprocess.run(["yt-dlp", *args], capture_output=True, text=True)
        if done.returncode == 0:
            return done.stdout
        if attempt + 1 < TRIES:
            time.sleep(2 ** attempt * 5)
    raise LookupFailed(done.stderr.strip().splitlines()[-1] if done.stderr.strip() else f"exit {done.returncode}")


def base(title, normalize):
    """The title without a bracketed English name or tag: "เธอ (Tur)" -> "เธอ"."""
    return normalize(re.split(r"\s*[(\[]", title, maxsplit=1)[0])


def find(song, normalize):
    """(video id, seconds) of the song's official audio track, or None."""
    query = urllib.parse.quote(f"{song['title']} {song['artist']}")
    first = run(["--flat-playlist", "--playlist-end", "1", "--print", "%(id)s\t%(title)s",
                 f"https://music.youtube.com/search?q={query}#songs"]).strip()
    if "\t" not in first:
        return None
    vid, title = first.split("\t", 1)
    wants = {base(song["title"], normalize)} | ({normalize(song["alias"])} if song.get("alias") else set())
    if base(title, normalize) not in wants or OTHER_VERSION.search(title):
        return None
    meta = run(["--skip-download", "--print", "%(duration)s\t%(track)s", "--", vid]).strip().split("\t")
    if len(meta) != 2 or meta[0] in ("NA", "") or meta[1] in ("NA", "") or OTHER_VERSION.search(meta[1]):
        return None
    secs = int(float(meta[0]))
    return (vid, secs) if GAP[0] <= song["secs"] - secs <= GAP[1] else None


def save(cache_path, cached):
    tmp = f"{cache_path}.tmp"
    with open(tmp, "w", encoding="utf-8") as f:
        json.dump(cached, f)
    os.replace(tmp, cache_path)


def find_all(songs, normalize, cache_path, pool, lookup=find):
    """{karaoke id: video id or None} for `songs`; results are cached (the lookup is two requests per song).

    A failed lookup is neither cached nor returned, so the song keeps whatever timing it has and is retried next run.
    The cache is saved every CHECKPOINT songs; after MAX_FAILED_IN_A_ROW failures the remaining songs are skipped."""
    cached = {}
    if cache_path and os.path.exists(cache_path):
        with open(cache_path, encoding="utf-8") as f:
            cached = json.load(f)
    todo = [s for s in songs if s["id"] not in cached]
    futures = {pool.submit(lookup, s, normalize): s for s in todo}
    failed, in_a_row = 0, 0
    try:
        for n, future in enumerate(as_completed(futures), 1):
            song = futures[future]
            try:
                hit = future.result()
            except CancelledError:
                continue
            except LookupFailed as e:
                failed, in_a_row = failed + 1, in_a_row + 1
                print(f"official audio: {song['id']} failed: {e}", file=sys.stderr)
                if in_a_row == MAX_FAILED_IN_A_ROW:
                    print(f"official audio: {in_a_row} failures in a row, skipping the rest", file=sys.stderr)
                    for f in futures:
                        f.cancel()
                continue
            in_a_row = 0
            cached[song["id"]] = hit[0] if hit else None
            if cache_path and n % CHECKPOINT == 0:
                save(cache_path, cached)
                print(f"official audio: {n}/{len(todo)} looked up, {failed} failed", file=sys.stderr)
    finally:
        if cache_path:
            save(cache_path, cached)
    return {s["id"]: cached[s["id"]] for s in songs if s["id"] in cached}
