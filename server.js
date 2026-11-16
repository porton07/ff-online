const http = require("http");
const fs = require("fs");
const path = require("path");
const crypto = require("crypto");

const PORT = process.env.PORT || 8080;
const WEB_DIR = path.join(__dirname, "web");

const MIME = {
  ".html": "text/html; charset=utf-8",
  ".js": "application/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".wasm": "application/wasm",
  ".png": "image/png",
  ".jpg": "image/jpeg",
  ".jpeg": "image/jpeg",
  ".wav": "audio/wav",
  ".json": "application/json",
};

// Minimal RFC 6455 WebSocket frame helpers (no external deps)
function encodeWsText(str) {
  const payload = Buffer.from(str, "utf8");
  const len = payload.length;
  let header;
  if (len < 126) {
    header = Buffer.alloc(2);
    header[0] = 0x81;
    header[1] = len;
  } else if (len < 65536) {
    header = Buffer.alloc(4);
    header[0] = 0x81;
    header[1] = 126;
    header.writeUInt16BE(len, 2);
  } else {
    header = Buffer.alloc(10);
    header[0] = 0x81;
    header[1] = 127;
    header.writeBigUInt64BE(BigInt(len), 2);
  }
  return Buffer.concat([header, payload]);
}

function attachWs(socket, onMessage, onClose) {
  let buf = Buffer.alloc(0);
  socket.on("data", (chunk) => {
    buf = Buffer.concat([buf, chunk]);
    while (buf.length >= 2) {
      const b0 = buf[0];
      const b1 = buf[1];
      const opcode = b0 & 0x0f;
      const masked = (b1 & 0x80) !== 0;
      let len = b1 & 0x7f;
      let off = 2;
      if (len === 126) {
        if (buf.length < 4) return;
        len = buf.readUInt16BE(2);
        off = 4;
      } else if (len === 127) {
        if (buf.length < 10) return;
        len = Number(buf.readBigUInt64BE(2));
        off = 10;
      }
      const maskLen = masked ? 4 : 0;
      if (buf.length < off + maskLen + len) return;
      const mask = masked ? buf.subarray(off, off + 4) : null;
      off += maskLen;
      const payload = Buffer.from(buf.subarray(off, off + len));
      buf = buf.subarray(off + len);
      if (masked) {
        for (let i = 0; i < payload.length; i++) payload[i] ^= mask[i & 3];
      }
      if (opcode === 0x8) {
        socket.end();
        return;
      }
      if (opcode === 0x9) {
        // Pong
        const pong = Buffer.alloc(2);
        pong[0] = 0x8a;
        pong[1] = 0;
        socket.write(pong);
        continue;
      }
      if (opcode === 0x1) {
        onMessage(payload.toString("utf8"));
      }
    }
  });
  socket.on("close", onClose);
  socket.on("error", onClose);
}

// Co-op Room management
const rooms = new Map();

function genCode() {
  const chars = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
  let code = "";
  for (let i = 0; i < 4; i++) code += chars[Math.floor(Math.random() * chars.length)];
  return rooms.has(code) ? genCode() : code;
}

function broadcastRoom(room) {
  const summary = JSON.stringify({
    type: "room_update",
    code: room.code,
    playerCount: room.clients.length,
  });
  const pkt = encodeWsText(summary);
  for (const c of room.clients) {
    if (!c.socket.destroyed) c.socket.write(pkt);
  }
}

const server = http.createServer((req, res) => {
  const reqPath = decodeURIComponent((req.url || "/").split("?")[0]);
  let filePath = path.join(WEB_DIR, reqPath === "/" ? "index.html" : reqPath);
  if (!filePath.startsWith(WEB_DIR)) {
    res.writeHead(403);
    res.end("Forbidden");
    return;
  }
  fs.stat(filePath, (err, st) => {
    if (err || !st.isFile()) {
      res.writeHead(404);
      res.end("Not Found");
      return;
    }
    const ext = path.extname(filePath).toLowerCase();
    res.writeHead(200, { "Content-Type": MIME[ext] || "application/octet-stream" });
    fs.createReadStream(filePath).pipe(res);
  });
});

server.on("upgrade", (req, socket) => {
  const key = req.headers["sec-websocket-key"];
  if (!key) {
    socket.destroy();
    return;
  }
  const accept = crypto
    .createHash("sha1")
    .update(key + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11")
    .digest("base64");
  socket.write(
    "HTTP/1.1 101 Switching Protocols\r\n" +
      "Upgrade: websocket\r\n" +
      "Connection: Upgrade\r\n" +
      `Sec-WebSocket-Accept: ${accept}\r\n\r\n`
  );

  let clientInfo = { socket, room: null, playerIdx: 0, isHost: false };

  attachWs(
    socket,
    (raw) => {
      let msg;
      try {
        msg = JSON.parse(raw);
      } catch {
        return;
      }

      if (msg.type === "create") {
        const code = genCode();
        const room = { code, host: clientInfo, clients: [clientInfo] };
        rooms.set(code, room);
        clientInfo.room = room;
        clientInfo.playerIdx = 0;
        clientInfo.isHost = true;
        socket.write(encodeWsText(JSON.stringify({ type: "created", code, playerIdx: 0 })));
        broadcastRoom(room);
      } else if (msg.type === "join") {
        const code = String(msg.code || "").trim().toUpperCase();
        const room = rooms.get(code);
        if (!room) {
          socket.write(encodeWsText(JSON.stringify({ type: "error", message: "Room not found" })));
          return;
        }
        if (room.clients.length >= 4) {
          socket.write(encodeWsText(JSON.stringify({ type: "error", message: "Room is full (max 4)" })));
          return;
        }
        clientInfo.room = room;
        clientInfo.playerIdx = room.clients.length;
        clientInfo.isHost = false;
        room.clients.push(clientInfo);
        socket.write(
          encodeWsText(JSON.stringify({ type: "joined", code, playerIdx: clientInfo.playerIdx }))
        );
        broadcastRoom(room);
      } else if (msg.type === "input" && clientInfo.room && !clientInfo.isHost) {
        const hostSock = clientInfo.room.host.socket;
        if (!hostSock.destroyed) {
          msg.playerIdx = clientInfo.playerIdx;
          hostSock.write(encodeWsText(JSON.stringify(msg)));
        }
      } else if (msg.type === "state" && clientInfo.room && clientInfo.isHost) {
        const pkt = encodeWsText(raw);
        for (const c of clientInfo.room.clients) {
          if (!c.isHost && !c.socket.destroyed) c.socket.write(pkt);
        }
      } else if (msg.type === "start_coop" && clientInfo.room && clientInfo.isHost) {
        const pkt = encodeWsText(raw);
        for (const c of clientInfo.room.clients) {
          if (!c.isHost && !c.socket.destroyed) c.socket.write(pkt);
        }
      }
    },
    () => {
      const room = clientInfo.room;
      if (!room) return;
      room.clients = room.clients.filter((c) => c !== clientInfo);
      if (clientInfo.isHost || room.clients.length === 0) {
        const closed = encodeWsText(JSON.stringify({ type: "room_closed" }));
        for (const c of room.clients) {
          if (!c.socket.destroyed) c.socket.write(closed);
          c.room = null;
        }
        rooms.delete(room.code);
      } else {
        room.clients.forEach((c, idx) => {
          c.playerIdx = idx;
        });
        broadcastRoom(room);
      }
    }
  );
});

server.listen(PORT, () => {
  console.log(`Feeding Frenzy Web + Co-Op Server running at http://localhost:${PORT}/`);
});
