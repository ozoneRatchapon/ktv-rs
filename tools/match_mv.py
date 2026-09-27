#!/usr/bin/env python3
"""Find the official original-vocal video (MV / lyric video) for the most-watched library songs.

Metadata only, like tools/harvest_library.py: `yt-dlp --flat-playlist` lists the karaoke channel (with view counts)
and the labels' official channels; nothing is downloaded. A candidate must name both the song title and the artist,
be an official MV / music video / lyric video (not a teaser, live, cover, remix, playlist...), run about as long as
the karaoke version, and embed (oEmbed, same rule as tools/link_check.py).

The timing (offset / rate) cannot be measured without the audio, which YouTube's terms rule out, so candidates start
unchecked: a curator lines each one up by ear in the app's Guide Timing Tools.

Writes tools/mv_candidates.json (every match, for review) and merges assets/mv_guides.json, the file the app reads:
{"<karaoke video id>": {"video_id": "<MV id>"}} is a suggestion (unchecked); an entry that also has "offset_secs" and
"rate" was lined up by a curator, is never replaced here, and gives the song its Vocal button.

Usage: python3 tools/match_mv.py [--top N] [--cache DIR]
       --cache DIR reuses/stores the channel listings as DIR/<name>.tsv (listing 30k videos takes minutes)
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
LIBRARY = os.path.join(ROOT, "assets", "library.json")
OUT = os.path.join(ROOT, "tools", "mv_candidates.json")
GUIDES = os.path.join(ROOT, "assets", "mv_guides.json")

KARAOKE = {"GMM Karaoke": ("karaoke_gmm", "https://www.youtube.com/channel/UCHmKRqvKPYVx23RJ8uF6AtA/videos")}
# Official channels per karaoke channel (the labels whose songs it carries)
MV_CHANNELS = {
    "GMM Karaoke": [
        ("mv_gmmgrammy", "https://www.youtube.com/@GMMGrammy/videos"),
        ("mv_genierock", "https://www.youtube.com/@genierock/videos"),
        ("mv_grammygold", "https://www.youtube.com/channel/UCFWsjsN53Sqvg2pWhSFmWBQ/videos"),
        # GMM sub-labels, found by searching the songs the three above missed
        ("mv_genelab", "https://www.youtube.com/channel/UCiJcM95iuhvQoGkmz30eV1w/videos"),
        ("mv_whitemusic", "https://www.youtube.com/channel/UCNdqGgAK2u-EucN4IAIqP7Q/videos"),
        ("mv_werecords", "https://www.youtube.com/channel/UCpvy4PMVogmhpE8jVjC9m_Q/videos"),
    ],
}

# A label's own channel tells a song's genre where the label keeps to one (names from catalog::CATEGORIES);
# GMM GRAMMY OFFICIAL and the other sub-labels mix genres, so their songs get none
GENRE_BY_CHANNEL = {"mv_grammygold": "Luk Thung", "mv_genierock": "Rock"}

# Best kind first; anything else is not a candidate
KINDS = [("mv", re.compile(r"official\s*m\.?v|music\s*video|【\s*m\.?v\s*】|\[\s*m\.?v\s*\]", re.I)),
         ("lyric", re.compile(r"lyric", re.I))]
NOT_ORIGINAL = re.compile(r"teaser|behind|longplay|long play|playlist|รวมเพลง|karaoke|คาราโอเกะ|โอเกะ|live|concert|"
                          r"คอนเสิร์ต|acoustic|cover|remix|instrumental|highlight|scoop|spot|demo|reaction|ver\.|version|"
                          r"re-mv|เกิดทัน|ติดเทรนด์|shorts|#", re.I)
THAI_MARKS = re.compile("[็-ํ]")


def normalize(text):
    """Lower-case letters and digits, Thai tone marks dropped (close to src/search/normalize.rs; Python also drops
    Thai vowel signs, harmless here since both sides of every comparison go through this)."""
    return "".join(c for c in THAI_MARKS.sub("", text).lower() if c.isalnum())


def listing(name, url, cache):
    path = os.path.join(cache, f"{name}.tsv") if cache else None
    if path and os.path.exists(path):
        with open(path, encoding="utf-8") as f:
            out = f.read()
    else:
        out = subprocess.run(["yt-dlp", "--flat-playlist", "--print", "%(id)s\t%(view_count)s\t%(duration)s\t%(title)s",
                              url], capture_output=True, text=True, check=True).stdout
        if path:
            os.makedirs(cache, exist_ok=True)
            with open(path, "w", encoding="utf-8") as f:
                f.write(out)
    rows = []
    for line in out.splitlines():
        parts = line.split("\t", 3)
        if len(parts) != 4:
            continue
        vid, views, secs, title = parts
        num = lambda v: int(float(v)) if v not in ("NA", "None", "") else 0  # noqa: E731
        rows.append({"id": vid, "views": num(views), "secs": num(secs), "title": title})
    return rows


def kind_of(title):
    if NOT_ORIGINAL.search(title):
        return None
    return next((kind for kind, pattern in KINDS if pattern.search(title)), None)


def artist_keys(artist):
    """The full artist, the lead artist without features ("X Feat. Y" -> "X"), and the lead's first word (MV titles
    spell names differently: "อัสนี วสันต์" vs "อัสนี โชติกุล, วสันต์ โชติกุล"). The song title must match as well."""
    main = re.split(r"\s*(?:\bfeat\.?|\bft\.?|\bx\b|&|,)\s*", artist, maxsplit=1, flags=re.I)[0]
    first = main.split()[0] if main.split() else main
    keys = {normalize(artist), normalize(main), normalize(first)}
    return {k for k in keys if len(k) >= 3}


def best_mv(song, mvs):
    # "คลื่น (Kleun)": the English alias in brackets is often missing from the MV title
    title = normalize(re.sub(r"\s*\([^)]*\)\s*$", "", song["title"]) or song["title"])
    if len(title) < 2:
        return None
    artists = artist_keys(song["artist"])
    found = []
    for mv in mvs:
        text = normalize(mv["title"])
        if title not in text or not any(a in text for a in artists):
            continue
        # MVs may add a story intro or outro; the timing offset absorbs it
        if not song["secs"] - 40 <= mv["secs"] <= song["secs"] + 200:
            continue
        rank = [kind for kind, _ in KINDS].index(mv["kind"])
        found.append((rank, -mv["views"], mv))
    return min(found, key=lambda f: f[:2])[2] if found else None


def genres_for(songs, mvs):
    """{karaoke video id: genre} for every song whose official video is on a single-genre label's channel."""
    genres = {}
    for song in songs:
        mv = best_mv(song, mvs)
        if mv and mv["channel"] in GENRE_BY_CHANNEL:
            genres[song["id"]] = GENRE_BY_CHANNEL[mv["channel"]]
    return genres


def write_genres(channels, cache):
    """Tag every library song (not only the most-watched) with its label's genre, in assets/library.json."""
    from harvest_library import CHANNELS, to_json
    label_genre = {c["name"]: c.get("genre") for c in CHANNELS}
    for channel in channels:
        channel["genre"] = label_genre.get(channel["name"])
        if channel["name"] not in MV_CHANNELS:
            continue
        mvs = [dict(mv, channel=mv_name, kind=kind_of(mv["title"]))
               for mv_name, url in MV_CHANNELS[channel["name"]] for mv in listing(mv_name, url, cache)]
        songs = [{"id": r[0], "secs": r[2], "title": r[3], "artist": r[4]} for r in channel["songs"]]
        channel["genres"] = genres_for(songs, [mv for mv in mvs if mv["kind"]])
        print(f"{channel['name']}: genre for {len(channel['genres'])} songs", file=sys.stderr)
    with open(LIBRARY, "w", encoding="utf-8") as f:
        f.write(to_json({"channels": channels}))


def merge_guides(guides, candidates):
    """Suggestions for the matched songs; timed entries (curated by ear) are kept as they are."""
    merged = {vid: g for vid, g in guides.items() if "offset_secs" in g}
    for c in candidates:
        merged.setdefault(c["karaoke_id"], {"video_id": c["video_id"]})
    return dict(sorted(merged.items()))


def guides_json(guides):
    """One song per line, so a re-run diffs as added / removed / timed songs."""
    rows = ",\n".join(f"  {json.dumps(k)}: {json.dumps(v, ensure_ascii=False, separators=(',', ':'))}"
                      for k, v in guides.items())
    return "{\n" + rows + "\n}\n"


def main():
    top = int(sys.argv[sys.argv.index("--top") + 1]) if "--top" in sys.argv else 200
    cache = sys.argv[sys.argv.index("--cache") + 1] if "--cache" in sys.argv else None
    with open(LIBRARY, encoding="utf-8") as f:
        channels = json.load(f)["channels"]
    candidates, stats = [], {}
    for channel in channels:
        name = channel["name"]
        if name not in KARAOKE or name not in MV_CHANNELS:
            continue
        views = {r["id"]: r["views"] for r in listing(*KARAOKE[name], cache)}
        mvs = [dict(mv, channel=mv_name, kind=kind_of(mv["title"]))
               for mv_name, url in MV_CHANNELS[name] for mv in listing(mv_name, url, cache)]
        mvs = [mv for mv in mvs if mv["kind"]]
        songs = [{"id": r[0], "code": r[1], "secs": r[2], "title": r[3], "artist": r[4], "views": views.get(r[0], 0)}
                 for r in channel["songs"]]
        songs.sort(key=lambda s: -s["views"])
        picked = songs[:top]
        matched = [(s, best_mv(s, mvs)) for s in picked]
        matched = [(s, mv) for s, mv in matched if mv]
        with ThreadPoolExecutor(max_workers=16) as pool:
            codes = list(pool.map(lambda pair: status(pair[1]["id"]), matched))
        embeddable = [(s, mv) for (s, mv), code in zip(matched, codes) if code == 200]
        stats[name] = {"top": len(picked), "matched": len(matched), "embeddable": len(embeddable), "mv_pool": len(mvs)}
        for s, mv in embeddable:
            candidates.append({
                "karaoke_id": s["id"], "code": s["code"], "title": s["title"], "artist": s["artist"],
                "karaoke_views": s["views"], "karaoke_secs": s["secs"],
                "video_id": mv["id"], "video_title": mv["title"], "video_channel": mv["channel"],
                "video_kind": mv["kind"], "video_views": mv["views"], "video_secs": mv["secs"],
            })
    with open(OUT, "w", encoding="utf-8") as f:
        json.dump({"stats": stats, "candidates": candidates}, f, ensure_ascii=False, indent=1)
        f.write("\n")
    write_genres(channels, cache)
    guides = {}
    if os.path.exists(GUIDES):
        with open(GUIDES, encoding="utf-8") as f:
            guides = json.load(f)
    merged = merge_guides(guides, candidates)
    with open(GUIDES, "w", encoding="utf-8") as f:
        f.write(guides_json(merged))
    timed = sum(1 for g in merged.values() if "offset_secs" in g)
    print(json.dumps(stats), f"assets/mv_guides.json: {len(merged)} songs, {timed} timed", file=sys.stderr)


if __name__ == "__main__":
    main()
