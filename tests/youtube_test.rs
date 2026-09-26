use app::youtube::{is_video_id, parse_video_id};

const ID: &str = "9aCUDQ8SPcA";

#[test]
fn test_accepts_bare_id_and_url_forms() {
    let inputs = [
        ID.to_string(),
        format!("  {ID}  "),
        format!("https://www.youtube.com/watch?v={ID}"),
        format!("https://www.youtube.com/watch?v={ID}&t=42s"),
        format!("https://www.youtube.com/watch?feature=share&v={ID}"),
        format!("https://m.youtube.com/watch?v={ID}#comments"),
        format!("https://music.youtube.com/watch?v={ID}&list=RD"),
        format!("https://youtu.be/{ID}"),
        format!("https://youtu.be/{ID}?si=abc"),
        format!("https://www.youtube.com/shorts/{ID}"),
        format!("https://www.youtube.com/embed/{ID}?start=10"),
        format!("https://www.youtube-nocookie.com/embed/{ID}"),
        format!("https://www.youtube.com/live/{ID}"),
    ];
    for input in inputs {
        assert_eq!(parse_video_id(&input).as_deref(), Some(ID), "{input}");
    }
}

#[test]
fn test_rejects_malformed_input() {
    let inputs = [
        "",
        "   ",
        "hello world",
        "9aCUDQ8SPc",   // 10 chars
        "9aCUDQ8SPcAx", // 12 chars
        "9aCUDQ8SP!A",
        "https://www.youtube.com/watch?v=short",
        "https://youtu.be/",
        "https://www.youtube.com/",
        "https://www.youtube.com/watch?v=9aCUDQ8SPcAxyz",
    ];
    for input in inputs {
        assert_eq!(parse_video_id(input), None, "{input:?}");
    }
}

#[test]
fn test_is_video_id_charset() {
    assert!(is_video_id("abc-DEF_123"));
    assert!(!is_video_id("abc DEF_123"));
    assert!(!is_video_id("ก".repeat(11).as_str()));
}
