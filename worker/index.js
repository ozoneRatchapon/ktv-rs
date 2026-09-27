// KTV-RS Worker. Static assets (the Dioxus build in ./dist) are served by Cloudflare before this script runs;
// only `/api/*` reaches it (`run_worker_first` in wrangler.jsonc). The one API is the phone remote's room socket.
import { ROOM_ID } from './protocol.js';

export { Room } from './room.js';

const text = (body, status) => new Response(body, { status, headers: { 'content-type': 'text/plain; charset=utf-8' } });

export default {
    /** `GET /api/room/<room>?role=booth|phone` with `Upgrade: websocket`, from a page on this site. */
    fetch(request, env) {
        const url = new URL(request.url);
        const room = url.pathname.match(/^\/api\/room\/([^/]+)$/)?.[1];
        if (!room || !ROOM_ID.test(room)) return text('Not found', 404);
        if (request.headers.get('Upgrade') !== 'websocket') return text('Expected a WebSocket', 426);
        // Another site's page cannot drive a room from a guest's browser
        if (request.headers.get('Origin') !== url.origin) return text('Forbidden', 403);
        const role = url.searchParams.get('role');
        if (role !== 'booth' && role !== 'phone') return text('Bad role', 400);
        return env.ROOM.get(env.ROOM.idFromName(room)).fetch(request);
    },
};
