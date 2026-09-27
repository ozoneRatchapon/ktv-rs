//! Plan 003 S3: a tip memo decides what the booth does (queue a request, or thank the singer).

use app::booth::{Booth, Placement, Requester};
use app::catalog::builtin_catalog;
use app::tip::{
    request_memo, request_page_url, song_memo, tip_action, ConfirmedTip, SolanaCluster, TipAction, MIN_REQUEST_TIP,
};

fn tip(amount: u64, memo: Option<&str>) -> ConfirmedTip {
    ConfirmedTip { signature: "sig".to_string(), amount, memo: memo.map(str::to_string) }
}

#[test]
fn a_request_memo_with_enough_usdc_queues_the_song() {
    let memo = request_memo("12345");
    assert_eq!(memo, "ktv:req:12345");
    assert_eq!(tip_action(&tip(MIN_REQUEST_TIP, Some(&memo))), TipAction::Request("12345".to_string()));
    assert_eq!(tip_action(&tip(5_000_000, Some(&memo))), TipAction::Request("12345".to_string()));
}

#[test]
fn dust_bad_codes_and_song_tips_only_get_a_garland() {
    assert_eq!(tip_action(&tip(MIN_REQUEST_TIP - 1, Some("ktv:req:12345"))), TipAction::Garland, "dust cannot jump the queue");
    assert_eq!(tip_action(&tip(1_000_000, Some("ktv:req:1234"))), TipAction::Garland, "not a 5-digit code");
    assert_eq!(tip_action(&tip(1_000_000, Some("ktv:req:12345; drop"))), TipAction::Garland);
    assert_eq!(tip_action(&tip(1_000_000, Some(&song_memo("12345")))), TipAction::Garland, "a tip on the song playing");
    assert_eq!(tip_action(&tip(1_000_000, None)), TipAction::Garland);
}

#[test]
fn the_request_page_link_carries_the_room_in_the_fragment() {
    let url = request_page_url(
        "https://ktv-rs.solana-thailand.workers.dev",
        "75AjMdh7Gn1TLigfze541AVJGJ4TyqBEaRZk3pozfBza",
        SolanaCluster::Devnet,
        "GVJJ7rdGiXr5xaYbRwRbjfaJL7fmwRygFi1H6aGqDveb",
        "VIP ห้อง 7",
    );
    assert_eq!(
        url,
        "https://ktv-rs.solana-thailand.workers.dev/request#to=75AjMdh7Gn1TLigfze541AVJGJ4TyqBEaRZk3pozfBza\
         &c=devnet&ref=GVJJ7rdGiXr5xaYbRwRbjfaJL7fmwRygFi1H6aGqDveb&room=VIP%20%E0%B8%AB%E0%B9%89%E0%B8%AD%E0%B8%87%207"
    );
    assert_eq!(SolanaCluster::Mainnet.slug(), "mainnet");
}

#[test]
fn a_tip_request_plays_next_and_is_labelled() {
    let songs = builtin_catalog();
    let mut booth = Booth::default();
    booth.add(songs[0].clone(), Requester::Singer, Placement::Now);
    booth.add(songs[1].clone(), Requester::Guest, Placement::Back);
    booth.add(songs[2].clone(), Requester::Tip, Placement::Next);
    assert_eq!(booth.queue[0].song.id, songs[2].id);
    assert_eq!(booth.queue[0].requester, "★ TIP");
}
