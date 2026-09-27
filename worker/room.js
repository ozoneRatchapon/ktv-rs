// One karaoke room: relays phone commands to the booth and the booth's state (now playing, up next) to phones.
// WebSocket Hibernation API: the object sleeps between messages, so an idle room costs nothing. Per-socket data
// lives in attachments (they survive hibernation); the only stored value is the last booth state, deleted by an
// alarm a day after the booth was last heard from. No names, no history.
import { DurableObject } from 'cloudflare:workers';
import { CLOSE, IDLE_MS, MAX_PENDING_BOOTHS, MAX_PHONES, parse_booth, parse_phone, room_of, take_token } from './protocol.js';

const STATE_KEY = 'state';
const OPEN = 1;

/** Send if the socket is still open: `getWebSockets()` also lists sockets that are closing, and sending to one throws. */
function send(ws, text) {
    if (ws.readyState !== OPEN) return;
    try {
        ws.send(text);
    } catch { /* closed between the check and the send */ }
}

export class Room extends DurableObject {
    /** Upgrade to a WebSocket. The Worker has checked the path, `role` and Origin. */
    async fetch(request) {
        const url = new URL(request.url);
        const role = url.searchParams.get('role');
        const room = url.pathname.split('/').pop();
        if (role === 'phone' && this.ctx.getWebSockets('phone').length >= MAX_PHONES) {
            return new Response('Room full', { status: 429 });
        }
        if (role === 'booth') {
            // A guest knows the room id (it is in the QR) but not the key: unproven booth sockets never pile up
            const pending = this.ctx.getWebSockets('booth').filter((b) => !b.deserializeAttachment().authed);
            for (const stale of pending.slice(0, Math.max(0, pending.length - MAX_PENDING_BOOTHS + 1))) {
                stale.close(CLOSE.bad_key, 'no room key');
            }
        }
        const [client, server] = Object.values(new WebSocketPair());
        this.ctx.acceptWebSocket(server, [role]);
        const id = crypto.getRandomValues(new Uint32Array(1))[0] >>> 1;
        server.serializeAttachment({ role, room, id, authed: false, bucket: null });
        if (role === 'phone') {
            const state = (await this.ctx.storage.get(STATE_KEY)) ?? null;
            send(server, JSON.stringify({ t: 'state', state, online: this.booth() !== null }));
            this.send_phone_count();
        }
        return new Response(null, { status: 101, webSocket: client });
    }

    webSocketMessage(ws, message) {
        const meta = ws.deserializeAttachment();
        if (meta.role === 'booth') return this.from_booth(ws, meta, message);
        return this.from_phone(ws, meta, message);
    }

    async from_booth(ws, meta, text) {
        const msg = parse_booth(text);
        if (!msg) return;
        if (!meta.authed) {
            if (msg.t !== 'hello' || (await room_of(msg.key)) !== meta.room) {
                ws.close(CLOSE.bad_key, 'wrong room key');
                return;
            }
            // One booth per room: a second tab (or a reconnect) takes over
            for (const other of this.ctx.getWebSockets('booth')) {
                if (other !== ws && other.deserializeAttachment().authed) other.close(CLOSE.replaced, 'another booth tab took over');
            }
            ws.serializeAttachment({ ...meta, authed: true });
            this.broadcast({ t: 'online', online: true });
            this.send_phone_count();
            return;
        }
        switch (msg.t) {
            case 'state': {
                const state = JSON.stringify(msg.state);
                await this.ctx.storage.put(STATE_KEY, msg.state);
                await this.ctx.storage.setAlarm(Date.now() + IDLE_MS);
                this.broadcast(`{"t":"state","online":true,"state":${state}}`);
                break;
            }
            case 'reply': {
                const phone = this.ctx.getWebSockets('phone').find((p) => p.deserializeAttachment().id === msg.to);
                if (phone) send(phone, JSON.stringify({ t: 'reply', ok: msg.ok, text: msg.text }));
                break;
            }
            case 'close_room':
                for (const phone of this.ctx.getWebSockets('phone')) phone.close(CLOSE.room_closed, 'the booth made a new link');
                await this.ctx.storage.deleteAll();
                break;
        }
    }

    from_phone(ws, meta, text) {
        const cmd = parse_phone(text);
        if (!cmd) return;
        const [allowed, bucket] = take_token(meta.bucket, Date.now());
        ws.serializeAttachment({ ...meta, bucket });
        const booth = this.booth();
        if (!allowed) return send(ws, JSON.stringify({ t: 'reply', ok: false, text: 'Too many requests: wait a few seconds' }));
        if (!booth) return send(ws, JSON.stringify({ t: 'reply', ok: false, text: 'The booth is offline' }));
        send(booth, JSON.stringify({ t: 'cmd', from: meta.id, ...cmd }));
    }

    webSocketClose(ws, code) {
        const meta = ws.deserializeAttachment();
        // Finish the close handshake (a no-op where the runtime already answered). 1005/1006 may not be sent.
        try {
            ws.close(code === 1005 || code === 1006 ? 1000 : code, 'closed');
        } catch { /* already closed */ }
        if (meta.role === 'phone') this.send_phone_count(ws);
        else if (meta.authed && !this.booth(ws)) this.broadcast({ t: 'online', online: false });
    }

    webSocketError(ws) {
        this.webSocketClose(ws, 1011);
    }

    /** Idle for a day: forget the room's state (phones reconnecting later see an empty room until the booth is back). */
    async alarm() {
        if (this.ctx.getWebSockets().length > 0) {
            await this.ctx.storage.setAlarm(Date.now() + IDLE_MS);
            return;
        }
        await this.ctx.storage.deleteAll();
    }

    /** The authenticated, open booth socket, ignoring `closing` (a socket that is closing is still listed). */
    booth(closing) {
        return this.ctx.getWebSockets('booth').find((b) => b !== closing && b.readyState === OPEN && b.deserializeAttachment().authed) ?? null;
    }

    send_phone_count(closing) {
        const n = this.ctx.getWebSockets('phone').filter((p) => p !== closing && p.readyState === OPEN).length;
        const booth = this.booth();
        if (booth) send(booth, JSON.stringify({ t: 'phones', n }));
    }

    broadcast(msg) {
        const text = typeof msg === 'string' ? msg : JSON.stringify(msg);
        for (const phone of this.ctx.getWebSockets('phone')) send(phone, text);
    }
}
