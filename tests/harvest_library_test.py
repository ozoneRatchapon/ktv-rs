"""Offline checks of tools/harvest_library.py title parsing and keypad-code rules.

Run: python3 -m unittest discover -s tests -p "*_test.py"
"""
import os
import sys
import unittest

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "tools"))
from harvest_library import assign_codes, parse_artist_first, parse_gmm, parse_title_first, retire_codes  # noqa: E402


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


if __name__ == "__main__":
    unittest.main()
