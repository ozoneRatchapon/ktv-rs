// Song search for the phone pages (/request, /remote): the booth's own index (`/songs.txt`, built at release from
// the same library, so codes always match), ranked the way the booth search ranks. Plain script, no wasm.
'use strict';

const KtvSongSearch = (() => {
    /** Search text as the booth compares it (src/search/normalize.rs): lower case, letters and digits only, Thai tone
     *  marks (U+0E47–U+0E4D) dropped. `\p{Alphabetic}` keeps Thai vowel signs, as Rust's `is_alphanumeric` does. */
    function normalize(text) {
        return text.toLowerCase().replace(/[็-ํ]/g, '').replace(/[^\p{Alphabetic}\p{N}]/gu, '');
    }

    /** `/songs.txt` lines (`code \t title \t artist \t aliases`) → songs with a search key per field. */
    function parse_songs(text) {
        return text.split('\n').filter(Boolean).map((line) => {
            const [code, title = '', artist = '', aliases = ''] = line.split('\t');
            return { code, title, artist, keys: [normalize(title), normalize(artist), normalize(aliases)] };
        });
    }

    /** Best matches first, as on the booth: code, title (start, inside), artist, aliases, then every word somewhere. */
    function find_songs(songs, query, limit = 30) {
        const needle = normalize(query);
        if (!needle) return [];
        const words = query.split(/\s+/).map(normalize).filter(Boolean);
        const rank = ({ code, keys: [title, artist, aliases] }) => {
            if (code === query.trim()) return 0;
            if (title.startsWith(needle)) return 1;
            if (title.includes(needle)) return 2;
            if (artist.includes(needle)) return 3;
            if (aliases.includes(needle)) return 4;
            if (words.length > 1 && words.every((w) => title.includes(w) || artist.includes(w) || aliases.includes(w))) return 5;
            return -1;
        };
        return songs
            .map((song, i) => ({ song, i, r: rank(song) }))
            .filter(({ r }) => r >= 0)
            .sort((a, b) => a.r - b.r || a.i - b.i)
            .slice(0, limit)
            .map(({ song }) => song);
    }

    /** List matches for `input` in `results` as buttons; a tap calls `on_pick(song)` and clears the list. The index
     *  loads on first use (≈200 KB compressed), so a guest who knows the code never pays for it. */
    function attach_search(input, results, on_pick) {
        let songs = null;
        const show = () => {
            results.replaceChildren(...find_songs(songs ?? [], input.value).map((song) => {
                const button = document.createElement('button');
                button.type = 'button';
                const code_span = Object.assign(document.createElement('span'), { className: 'code', textContent: song.code });
                const artist_span = Object.assign(document.createElement('span'), { className: 'artist', textContent: song.artist });
                button.append(code_span, song.title, artist_span);
                button.lang = 'th';
                button.addEventListener('click', () => {
                    input.value = song.title;
                    results.replaceChildren();
                    on_pick(song);
                });
                const item = document.createElement('li');
                item.append(button);
                return item;
            }));
        };
        input.addEventListener('input', async () => {
            if (!songs) {
                songs = [];
                const response = await fetch('/songs.txt').catch(() => null);
                songs = response?.ok ? parse_songs(await response.text()) : [];
            }
            show();
        });
    }

    return { attach_search, find_songs, normalize, parse_songs };
})();

if (typeof module !== 'undefined') module.exports = KtvSongSearch;
