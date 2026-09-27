"""Offline checks of tools/harvest_library.py title parsing and keypad-code rules.

Run: python3 -m unittest discover -s tests -p "*_test.py"
"""
import os
import sys
import unittest

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "tools"))
from harvest_library import (  # noqa: E402
    assign_codes, parse_artist_first, parse_gmm, parse_smallroom, parse_title_colon, parse_title_first, retire_codes)


class ParseGmm(unittest.TestCase):
    def test_romanised_alias(self):
        self.assertEqual(
            parse_gmm("คาราโอเกะ อิจฉาภรรยาอ้าย (It-Cha-Pan-Ra-Ya-Ai) - ศิริพร อำไพพงษ์ [ Original Karaoke ]"),
            ("อิจฉาภรรยาอ้าย", "ศิริพร อำไพพงษ์", "It Cha Pan Ra Ya Ai"))

    def test_trailing_tag_variants_leave_the_artist(self):
        for title in [
            "คาราโอเกะ ขอโทษ - COCKTAIL Original Karaoke",
            "คาราโอเกะ ขอโทษ - COCKTAIL [ Original Karaoke",
            "คาราโอเกะ ขอโทษ - COCKTAIL  Original Karaoke ]",
            "คาราโอเกะ ขอโทษ - COCKTAIL [ Original Karaoke }",
            "คาราโอเกะ ขอโทษ - COCKTAIL [ 3 รอบ ] ( Original Karaoke )",
            "คาราโอเกะ ขอโทษ - COCKTAIL [ Original Karaokeo ]",
        ]:
            self.assertEqual(parse_gmm(title), ("ขอโทษ", "COCKTAIL", ""), title)

    def test_separator_missing_a_space(self):
        self.assertEqual(parse_gmm("คาราโอเกะ เธอคงไม่รู้ -ZAZA"), ("เธอคงไม่รู้", "ZAZA", ""))
        self.assertEqual(parse_gmm("คาราโอเกะ เด็กเกินไป- Three Man Down"), ("เด็กเกินไป", "Three Man Down", ""))
        # a dash inside a word is not a separator, and the romanised title keeps its own
        self.assertIsNone(parse_gmm("คาราโอเกะ อยู่ไป-ไม่มีเธอ BLACKHEAD"))
        self.assertEqual(
            parse_gmm("คาราโอเกะ ลาก่อน (La-Gone) Par- T [Original Karaoke]"), ("ลาก่อน", "Par- T", "La Gone"))

    def test_event_tag_is_dropped(self):
        self.assertEqual(
            parse_gmm("คาราโอเกะ หัวใจสะออน (ซนซน 40 ปี GMM GRAMMY)- โจอี้ ภูวศิษฐ์"), ("หัวใจสะออน", "โจอี้ ภูวศิษฐ์", ""))

    def test_no_separator_is_skipped(self):
        self.assertIsNone(parse_gmm("คาราโอเกะ ปวดใจ I-ZAX"))


class ParseOtherLabels(unittest.TestCase):
    def test_title_first(self):
        self.assertEqual(parse_title_first("ชื่อเพลง - ศิลปิน [คาราโอเกะ]"), ("ชื่อเพลง", "ศิลปิน", ""))

    def test_artist_first(self):
        self.assertEqual(parse_artist_first("ศิลปิน - ชื่อเพลง [Official Karaoke]"), ("ชื่อเพลง", "ศิลปิน", ""))

    def test_rs_title_colon(self):
        self.assertEqual(parse_title_colon("ใจเหลือเหลือ : Dr.fuu [Official Karaoke]"), ("ใจเหลือเหลือ", "Dr.fuu", ""))
        self.assertEqual(parse_title_colon("คนเก่ง : เต๋า สมชาย [Official karaoke]"), ("คนเก่ง", "เต๋า สมชาย", ""))
        self.assertIsNone(parse_title_colon("NA"))

    def test_smallroom(self):
        self.assertEqual(parse_smallroom("MOOR - แต่งงาน | Will you marry me? [Karaoke]"), ("แต่งงาน", "MOOR", "Will you marry me?"))
        self.assertEqual(parse_smallroom("SLUR – เพราะทุกครั้ง | TEARS [Karaoke]"), ("เพราะทุกครั้ง", "SLUR", "TEARS"))
        self.assertEqual(parse_smallroom('yarinda "เพ่ง" [Karaoke]'), ("เพ่ง", "yarinda", ""))
        self.assertEqual(parse_smallroom("LEMONSOUP - บางคน [Karaoke]"), ("บางคน", "LEMONSOUP", ""))


class Codes(unittest.TestCase):
    def test_kept_videos_keep_codes_and_retired_codes_are_never_reused(self):
        old_codes = {"kept": 30001, "gone": 30002, "retired": 30003}
        songs = [{"vid": "new"}, {"vid": "kept"}]
        assign_codes(songs, (30001, 30010), old_codes)
        self.assertEqual({s["vid"]: s["code"] for s in songs}, {"new": 30004, "kept": 30001})

    def test_returning_video_gets_its_code_back(self):
        songs = [{"vid": "retired"}]
        assign_codes(songs, (30001, 30010), {"retired": 30003})
        self.assertEqual(songs[0]["code"], 30003)

    def test_retire_codes(self):
        retired = {30003: "back"}
        old_codes = {"kept": 30001, "gone": 30002, "back": 30003}
        library = {"channels": [{"songs": [["kept", 30001], ["back", 30003]]}]}
        retire_codes(old_codes, library, retired)
        self.assertEqual(retired, {30002: "gone"})



class MvGuides(unittest.TestCase):
    def test_merge_keeps_timed_entries_and_adds_suggestions(self):
        from match_mv import merge_guides
        guides = {"timed": {"video_id": "AAAAAAAAAAA", "offset_secs": 2.5, "rate": 1.0},
                  "stale": {"video_id": "BBBBBBBBBBB"}}
        candidates = [{"karaoke_id": "timed", "video_id": "CCCCCCCCCCC"}, {"karaoke_id": "new", "video_id": "DDDDDDDDDDD"}]
        merged = merge_guides(guides, candidates)
        self.assertEqual(merged["timed"]["video_id"], "AAAAAAAAAAA", "a curated timing is never replaced")
        self.assertEqual(merged["new"], {"video_id": "DDDDDDDDDDD"})
        self.assertNotIn("stale", merged, "a suggestion no longer matched is dropped")

    def test_artist_and_title_both_required(self):
        from match_mv import best_mv
        mvs = [{"title": "แพ้ใจ - ใหม่ เจริญปุระ 【OFFICIAL MV】", "secs": 250, "views": 10, "kind": "mv"},
               {"title": "แพ้ใจ - คนอื่น 【OFFICIAL MV】", "secs": 250, "views": 99, "kind": "mv"}]
        song = {"title": "แพ้ใจ", "artist": "ใหม่ เจริญปุระ", "secs": 250}
        self.assertEqual(best_mv(song, mvs)["title"], mvs[0]["title"])
        self.assertIsNone(best_mv({"title": "แพ้ใจ", "artist": "ศิลปินอื่น", "secs": 250}, mvs))


class LibraryJson(unittest.TestCase):
    def test_to_json_round_trips_genres(self):
        import json
        from harvest_library import to_json
        library = {"channels": [
            {"name": "A", "intro_skip_secs": 18, "genres": {"v2": "Rock", "v1": "Luk Thung"},
             "songs": [["v1", 30001, 200, "t", "a", ""], ["v2", 30002, 210, "u", "b", ""]]},
            {"name": "B", "intro_skip_secs": 0, "genre": "Indie", "genres": {}, "songs": [["v3", 65001, 190, "x", "y", ""]]},
        ]}
        back = json.loads(to_json(library))
        self.assertEqual(back["channels"][0]["genres"], {"v1": "Luk Thung", "v2": "Rock"})
        self.assertNotIn("genre", back["channels"][0])
        self.assertEqual(back["channels"][1]["genre"], "Indie")
        self.assertNotIn("genres", back["channels"][1], "empty maps are left out")
        self.assertEqual(to_json(back), to_json(library), "stable output")


class OfficialAudioMerge(unittest.TestCase):
    def test_audio_timing_beats_suggestions_but_never_hand_timing(self):
        from match_mv import merge_guides
        guides = {"hand": {"video_id": "HHHHHHHHHHH", "offset_secs": -17.9, "rate": 1.0},
                  "old_auto": {"video_id": "OOOOOOOOOOO", "offset_secs": -18.2, "rate": 1.0, "auto": True},
                  "lost_auto": {"video_id": "LLLLLLLLLLL", "offset_secs": -18.2, "rate": 1.0, "auto": True}}
        candidates = [{"karaoke_id": "new", "video_id": "MVMVMVMVMVM"}, {"karaoke_id": "hand", "video_id": "MVMVMVMVMV2"}]
        audio = {"new": "AAAAAAAAAAA", "hand": "BBBBBBBBBBB", "lost_auto": None}
        merged = merge_guides(guides, candidates, audio)
        self.assertEqual(merged["hand"]["video_id"], "HHHHHHHHHHH", "a curator's timing wins")
        self.assertEqual(merged["new"], {"video_id": "AAAAAAAAAAA", "offset_secs": -18.2, "rate": 1.0, "auto": True})
        self.assertIn("old_auto", merged, "not re-checked this run: kept")
        self.assertNotIn("lost_auto", merged, "re-checked and no longer found: dropped")

    def test_title_base_drops_bracketed_names(self):
        from match_mv import normalize
        from official_audio import OTHER_VERSION, base
        self.assertEqual(base("เธอ (Tur)", normalize), normalize("เธอ"))
        self.assertTrue(OTHER_VERSION.search("ไม่มีครั้งสุดท้าย (อคูสติค เวอร์ชั่น)"))
        self.assertFalse(OTHER_VERSION.search("Nok Long Rung"))


if __name__ == "__main__":
    unittest.main()
