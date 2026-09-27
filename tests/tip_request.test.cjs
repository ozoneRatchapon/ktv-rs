// Unit tests for the phone request page (public/request.js). Run: node --test tests/tip_request.test.cjs
'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { USDC_MINT, find_songs, normalize, parse_room, parse_songs, request_link } = require('../public/request.js');

const WALLET = '75AjMdh7Gn1TLigfze541AVJGJ4TyqBEaRZk3pozfBza';
const REFERENCE = 'GVJJ7rdGiXr5xaYbRwRbjfaJL7fmwRygFi1H6aGqDveb';
const hash = (c = 'devnet') => `#to=${WALLET}&c=${c}&ref=${REFERENCE}&room=%E0%B8%AB%E0%B9%89%E0%B8%AD%E0%B8%87%201`;

test('mints match the Rust side (src/tip/types.rs)', () => {
    const rust = fs.readFileSync(path.join(__dirname, '../src/tip/types.rs'), 'utf8');
    assert.ok(rust.includes(`Self::Devnet => "${USDC_MINT.devnet}"`));
    assert.ok(rust.includes(`Self::Mainnet => "${USDC_MINT.mainnet}"`));
});

test('the room comes from the fragment and must be complete', () => {
    assert.deepEqual(parse_room(hash()), { wallet: WALLET, cluster: 'devnet', reference: REFERENCE, name: 'ห้อง 1' });
    assert.equal(parse_room(''), null);
    assert.equal(parse_room(hash('testnet')), null, 'unknown cluster');
    assert.equal(parse_room(hash().replace(REFERENCE, '0OIl')), null, 'reference not base58');
    assert.equal(parse_room(`#c=devnet&ref=${REFERENCE}`), null, 'no wallet');
});

test('the link is a Solana Pay transfer with the request memo', () => {
    const link = request_link(parse_room(hash('mainnet')), '12345', '2');
    assert.equal(link,
        `solana:${WALLET}?amount=2&spl-token=${USDC_MINT.mainnet}&reference=${REFERENCE}`
        + '&label=%E0%B8%AB%E0%B9%89%E0%B8%AD%E0%B8%87%201&message=Request%20%2312345&memo=ktv%3Areq%3A12345');
});

test('normalize matches the booth (src/search/normalize.rs tests)', () => {
    assert.equal(normalize('รักไม่ไหว'), normalize('รักไมไหว'), 'tone marks dropped');
    assert.equal(normalize('Rak-Mai Wai'), 'rakmaiwai');
    assert.equal(normalize('ยิ่งใกล้...ยิ่งไกล'), normalize('ยิงใกล ยิงไกล'));
    assert.equal(normalize('สั้น'), 'สัน', 'vowel signs kept, tone mark dropped');
});

test('phone search ranks like the booth and finds artist + title typed together', () => {
    const songs = parse_songs([
        '10001\tรักแล้วยอมได้\tแช่ม แช่มรัมย์\tRak Laew Yom Dai',
        '10002\tเพลงรัก\tศิลปินหนึ่ง\t',
        '10004\tรักไม่ไหวแล้วโว้ย\tโจอี้ ภูวศิษฐ์\tRak Mai Wai Laew Woi',
        '10005\tคนที่รัก\tรักษ์ วงดนตรี\t',
        '',
    ].join('\n'));
    assert.equal(songs.length, 4);
    const codes = (q) => find_songs(songs, q).map((s) => s.code);
    assert.deepEqual(codes('รัก'), ['10001', '10004', '10002', '10005'], 'title starts, then title contains, list order kept');
    assert.deepEqual(codes('รักไมไหว'), ['10004']);
    assert.deepEqual(codes('rak mai wai'), ['10004'], 'romanised alias');
    assert.deepEqual(codes('ภูวศิษฐ์ รักไม่ไหว'), ['10004'], 'artist + title');
    assert.deepEqual(codes('10002'), ['10002'], 'code');
    assert.deepEqual(codes('   '), []);
    assert.equal(find_songs(songs, 'รัก', 2).length, 2, 'limit');
});
