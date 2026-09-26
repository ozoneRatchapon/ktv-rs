#!/usr/bin/env python3
"""Measure GuideTrack timing (MV time = offset_secs + rate * karaoke time).

Downloads both audio tracks, builds chroma features (robust to the extra vocals
in the MV), cross-correlates 30s karaoke windows against the MV, then fits a
line through the confident matches with outlier rejection.

Usage (needs yt-dlp, ffmpeg, numpy):
    python3 tools/guide_align.py [--write] <karaoke_youtube_id> <guide_youtube_id> [...more pairs]

Output: offset_secs / rate for the song's `guide` in assets/catalog.json.
`null` means the guide does not match the karaoke track (wrong video/version, or an
edited MV whose timing breaks part-way through).
--write: store each successful fit as the `guide` of the catalog song whose `youtube_id`
is the karaoke id (a failed fit leaves the entry untouched). Run `cargo test` afterwards.
"""
import json
import os
import subprocess
import sys
import tempfile

import numpy as np

SR = 16000
FPS = 20
CACHE = os.path.join(tempfile.gettempdir(), "ktv-guide-align")
CATALOG = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "assets", "catalog.json")


def fetch(video_id):
    os.makedirs(CACHE, exist_ok=True)
    wav = os.path.join(CACHE, f"{video_id}.wav")
    if not os.path.exists(wav):
        subprocess.run(
            ["yt-dlp", "-q", "--no-warnings", "--retries", "5", "-f", "ba/91/92/93/94/b",  # HLS fallback: dash audio sometimes 403s
             "-o", os.path.join(CACHE, f"{video_id}.%(ext)s"),
             "--exec", f"ffmpeg -loglevel error -y -i {{}} -vn -ac 1 -ar {SR} {wav} && rm {{}}",
             f"https://www.youtube.com/watch?v={video_id}"],
            check=True,
        )
    raw = subprocess.run(["ffmpeg", "-loglevel", "error", "-i", wav, "-f", "s16le", "-"],
                         capture_output=True, check=True).stdout
    return np.frombuffer(raw, np.int16).astype(np.float32) / 32768


def chroma(x):
    n, hop = 4096, SR // FPS
    frames = np.lib.stride_tricks.sliding_window_view(x, n)[::hop] * np.hanning(n)
    mag = np.abs(np.fft.rfft(frames, axis=1))
    freqs = np.fft.rfftfreq(n, 1 / SR)
    sel = (freqs > 60) & (freqs < 2000)
    pitch_class = (np.round(12 * np.log2(freqs[sel] / 440)) % 12).astype(int)
    c = np.zeros((len(frames), 12))
    for b in range(12):
        c[:, b] = mag[:, sel][:, pitch_class == b].sum(1)
    c = np.log1p(c)
    c -= c.mean(1, keepdims=True)
    return c / (np.linalg.norm(c, axis=1, keepdims=True) + 1e-9)


def xcorr_window(k, m, a, b, max_lag):
    w, length = k[a:b], b - a
    size = 1 << (len(m) + length).bit_length()
    acc = np.zeros(size)
    for j in range(12):
        acc += np.fft.irfft(np.fft.rfft(m[:, j], size) * np.conj(np.fft.rfft(w[:, j], size)), size)
    lags = np.arange(-max_lag, max_lag + 1)
    starts = a + lags
    valid = (starts >= 0) & (starts + length <= len(m))
    return lags, np.where(valid, acc[starts % size] / length, -np.inf)


def measure(k, m, win=30, step=10, max_lag_s=150, start_s=18):
    points = []
    for t0 in range(start_s, int(len(k) / FPS) - win, step):
        lags, vals = xcorr_window(k, m, t0 * FPS, (t0 + win) * FPS, max_lag_s * FPS)
        finite = np.isfinite(vals)
        if finite.sum() < 50:
            continue
        i = int(np.nanargmax(np.where(finite, vals, np.nan)))
        far = vals[finite & (np.abs(lags - lags[i]) > 3 * FPS)]
        z = (vals[i] - np.median(far)) / (far.std() + 1e-9)
        d = 0.0
        if 0 < i < len(vals) - 1 and np.isfinite(vals[i - 1]) and np.isfinite(vals[i + 1]):
            den = vals[i - 1] - 2 * vals[i] + vals[i + 1]
            if den < 0:
                d = 0.5 * (vals[i - 1] - vals[i + 1]) / den
        points.append((t0 + win / 2, (lags[i] + d) / FPS, float(z)))
    return points


def fit(k, m, zmin=4.0):
    pts = [(t, o) for t, o, z in measure(k, m) if z >= zmin]
    if len(pts) < 4:
        return None
    t = np.array([p[0] for p in pts])
    o = np.array([p[1] for p in pts])
    keep = np.abs(o - np.median(o)) < 2.0
    for _ in range(5):
        if keep.sum() < 4:
            return None
        a, b = np.polyfit(t[keep], o[keep], 1)
        resid = np.abs(o - (a * t + b))
        keep = resid < max(0.3, 3 * np.median(resid[keep]))
    # Inliers must span the song: an edited MV (inserted scene) matches only up to the cut.
    coverage = float((t[keep].max() - t[keep].min()) / (t.max() - t.min()))
    if coverage < 0.8:
        return None
    a, b = np.polyfit(t[keep], o[keep], 1)
    rms = float(np.sqrt(np.mean((o[keep] - (a * t[keep] + b)) ** 2)))
    rate = 1 + a
    if abs(a) * 300 < 0.15:  # < 0.15s drift over 5 minutes -> constant offset
        rate, b = 1.0, float(np.median(o[keep]))
    return {"offset_secs": round(float(b), 2), "rate": round(float(rate), 4),
            "inliers": int(keep.sum()), "windows": len(pts), "rms_secs": round(rms, 3),
            "coverage": round(coverage, 2)}


def write_guides(fits):
    """Apply {karaoke_id: (guide_id, fit)} to assets/catalog.json, keeping serde_json's pretty format."""
    with open(CATALOG, encoding="utf-8") as f:
        catalog = json.load(f)
    by_karaoke = {song["youtube_id"]: song for song in catalog}
    for karaoke_id, (guide_id, result) in fits.items():
        song = by_karaoke.get(karaoke_id)
        if song is None:
            print(karaoke_id, "SKIP not in catalog", file=sys.stderr)
            continue
        song["guide"] = {"video_id": guide_id, "offset_secs": result["offset_secs"], "rate": result["rate"]}
        print(karaoke_id, "->", song["code"], song["title"], song["guide"], file=sys.stderr)
    tmp = CATALOG + ".tmp"
    with open(tmp, "w", encoding="utf-8") as f:
        f.write(json.dumps(catalog, indent=2, ensure_ascii=False) + "\n")
    os.replace(tmp, CATALOG)


def main():
    args = sys.argv[1:]
    write = "--write" in args
    ids = [a for a in args if a != "--write"]
    if not ids or len(ids) % 2:
        sys.exit(__doc__)
    fits = {}
    for karaoke_id, guide_id in zip(ids[::2], ids[1::2]):
        if karaoke_id == guide_id:
            print(karaoke_id, guide_id, "ERROR guide must differ from karaoke video")
            continue
        result = fit(chroma(fetch(karaoke_id)), chroma(fetch(guide_id)))
        print(karaoke_id, guide_id, json.dumps(result), flush=True)
        if result is not None:
            fits[karaoke_id] = (guide_id, result)
    if write and fits:
        write_guides(fits)


if __name__ == "__main__":
    main()
