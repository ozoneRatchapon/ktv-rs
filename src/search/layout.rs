/// Thai Kedmanee layout: the Thai character each US QWERTY key types (unshifted, then shifted keys).
const KEDMANEE: [(char, char); 94] = [
    ('`', '_'), ('1', 'ๅ'), ('2', '/'), ('3', '-'), ('4', 'ภ'), ('5', 'ถ'), ('6', 'ุ'), ('7', 'ึ'),
    ('8', 'ค'), ('9', 'ต'), ('0', 'จ'), ('-', 'ข'), ('=', 'ช'),
    ('q', 'ๆ'), ('w', 'ไ'), ('e', 'ำ'), ('r', 'พ'), ('t', 'ะ'), ('y', 'ั'), ('u', 'ี'), ('i', 'ร'),
    ('o', 'น'), ('p', 'ย'), ('[', 'บ'), (']', 'ล'), ('\\', 'ฃ'),
    ('a', 'ฟ'), ('s', 'ห'), ('d', 'ก'), ('f', 'ด'), ('g', 'เ'), ('h', '้'), ('j', '่'), ('k', 'า'),
    ('l', 'ส'), (';', 'ว'), ('\'', 'ง'),
    ('z', 'ผ'), ('x', 'ป'), ('c', 'แ'), ('v', 'อ'), ('b', 'ิ'), ('n', 'ื'), ('m', 'ท'), (',', 'ม'),
    ('.', 'ใ'), ('/', 'ฝ'),
    ('~', '%'), ('!', '+'), ('@', '๑'), ('#', '๒'), ('$', '๓'), ('%', '๔'), ('^', 'ู'), ('&', '฿'),
    ('*', '๕'), ('(', '๖'), (')', '๗'), ('_', '๘'), ('+', '๙'),
    ('Q', '๐'), ('W', '"'), ('E', 'ฎ'), ('R', 'ฑ'), ('T', 'ธ'), ('Y', 'ํ'), ('U', '๊'), ('I', 'ณ'),
    ('O', 'ฯ'), ('P', 'ญ'), ('{', 'ฐ'), ('}', ','), ('|', 'ฅ'),
    ('A', 'ฤ'), ('S', 'ฆ'), ('D', 'ฏ'), ('F', 'โ'), ('G', 'ฌ'), ('H', '็'), ('J', '๋'), ('K', 'ษ'),
    ('L', 'ศ'), (':', 'ซ'), ('"', '.'),
    ('Z', '('), ('X', ')'), ('C', 'ฉ'), ('V', 'ฮ'), ('B', 'ฺ'), ('N', '์'), ('M', '?'), ('<', 'ฒ'),
    ('>', 'ฬ'), ('?', 'ฦ'),
];

fn is_thai(c: char) -> bool {
    ('\u{0E00}'..='\u{0E7F}').contains(&c)
}

/// The query as it would have come out on the other layout: QWERTY keystrokes read as Thai when the
/// query has no Thai letters, Thai read back as QWERTY keys otherwise. `None` when nothing changes.
pub fn retype(query: &str) -> Option<String> {
    let to_thai = !query.chars().any(is_thai);
    let swapped: String = query
        .chars()
        .map(|c| {
            let hit = match to_thai {
                true => KEDMANEE.iter().find(|(key, _)| *key == c).map(|(_, thai)| *thai),
                false => KEDMANEE.iter().find(|(_, thai)| *thai == c).map(|(key, _)| *key),
            };
            hit.unwrap_or(c)
        })
        .collect();
    (swapped != query).then_some(swapped)
}
