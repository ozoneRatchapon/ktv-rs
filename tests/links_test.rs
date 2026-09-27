use app::catalog::builtin_catalog;
use app::links::{chords_search_url, percent_encode};

#[test]
fn test_chords_search_url_encodes_thai_title_and_artist() {
    let song = app::types::Song { title: "รัก ไม่ไหว".to_string(), artist: "A&B".to_string(), ..builtin_catalog()[0].clone() };
    let url = chords_search_url(&song);
    assert!(url.starts_with("https://www.google.com/search?q="), "{url}");
    let query = url.split_once("?q=").unwrap().1;
    assert!(query.bytes().all(|b| b.is_ascii_alphanumeric() || b"%-_.~".contains(&b)), "fully encoded: {query}");
    assert_eq!(query, percent_encode("คอร์ด รัก ไม่ไหว A&B"));
}
