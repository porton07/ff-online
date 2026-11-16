use serde::{Deserialize, Serialize};
use worker::*;

#[derive(Serialize, Deserialize, Clone, Default)]
struct PeerMeta {
    player_idx: usize,
    is_host: bool,
    code: String,
    fish_idx: usize,
    name: String,
}

#[derive(Serialize)]
struct RosterEntry {
    #[serde(rename = "playerIdx")]
    player_idx: usize,
    #[serde(rename = "fishIdx")]
    fish_idx: usize,
    name: String,
}

fn collect_roster(state: &State) -> Vec<RosterEntry> {
    let mut items: Vec<PeerMeta> = state
        .get_websockets()
        .into_iter()
        .filter_map(|ws| ws.deserialize_attachment::<PeerMeta>().ok().flatten())
        .collect();
    items.sort_by_key(|m| m.player_idx);
    items
        .into_iter()
        .map(|m| RosterEntry {
            player_idx: m.player_idx,
            fish_idx: m.fish_idx,
            name: if m.name.is_empty() {
                format!("Player {}", m.player_idx + 1)
            } else {
                m.name
            },
        })
        .collect()
}

#[durable_object]
pub struct CoopRoom {
    state: State,
    #[allow(dead_code)]
    env: Env,
}

impl DurableObject for CoopRoom {
    fn new(state: State, env: Env) -> Self {
        Self { state, env }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        let url = req.url()?;
        let mut role = "join".to_string();
        let mut code = "ROOM".to_string();
        for (k, v) in url.query_pairs() {
            if k == "role" {
                role = v.to_string();
            } else if k == "code" {
                code = v.to_string().to_uppercase();
            }
        }

        let existing = self.state.get_websockets();
        if role == "join" && existing.is_empty() {
            let pair = WebSocketPair::new()?;
            self.state.accept_web_socket(&pair.server);
            let _ = pair.server.send_with_str(
                r#"{"type":"error","message":"Room not found"}"#,
            );
            let _ = pair.server.close(Some(1000), Some("Room not found"));
            return Response::from_websocket(pair.client);
        }

        if existing.len() >= 4 {
            let pair = WebSocketPair::new()?;
            self.state.accept_web_socket(&pair.server);
            let _ = pair.server.send_with_str(
                r#"{"type":"error","message":"Room is full (max 4)"}"#,
            );
            let _ = pair.server.close(Some(1000), Some("Room full"));
            return Response::from_websocket(pair.client);
        }

        let is_host = existing.is_empty() && role == "host";
        let player_idx = if is_host { 0 } else { existing.len() };

        let pair = WebSocketPair::new()?;
        let server = pair.server;
        self.state.accept_web_socket(&server);

        let meta = PeerMeta {
            player_idx,
            is_host,
            code: code.clone(),
            fish_idx: player_idx % 5,
            name: format!("Player {}", player_idx + 1),
        };
        let _ = server.serialize_attachment(&meta);

        let welcome = if is_host {
            serde_json::json!({ "type": "created", "code": code, "playerIdx": 0 })
        } else {
            serde_json::json!({ "type": "joined", "code": code, "playerIdx": player_idx })
        };
        let _ = server.send_with_str(&welcome.to_string());

        let roster = collect_roster(&self.state);
        let update = serde_json::json!({
            "type": "room_update",
            "code": code,
            "playerCount": roster.len(),
            "roster": roster
        })
        .to_string();
        for ws in self.state.get_websockets() {
            let _ = ws.send_with_str(&update);
        }

        Response::from_websocket(pair.client)
    }

    async fn websocket_message(
        &self,
        ws: WebSocket,
        message: WebSocketIncomingMessage,
    ) -> Result<()> {
        let WebSocketIncomingMessage::String(raw) = message else {
            return Ok(());
        };
        let Ok(Some(meta)) = ws.deserialize_attachment::<PeerMeta>() else {
            return Ok(());
        };
        let Ok(mut val) = serde_json::from_str::<serde_json::Value>(&raw) else {
            return Ok(());
        };

        let msg_type = val.get("type").and_then(|v| v.as_str()).unwrap_or("");
        if msg_type == "pick_fish" || msg_type == "set_name" {
            let mut updated_meta = meta.clone();
            if msg_type == "pick_fish" {
                let fi = val.get("fishIdx").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                updated_meta.fish_idx = fi.min(4);
            } else if let Some(nm) = val.get("name").and_then(|v| v.as_str()) {
                let trimmed: String = nm.trim().chars().take(12).collect();
                updated_meta.name = if trimmed.is_empty() {
                    format!("Player {}", meta.player_idx + 1)
                } else {
                    trimmed
                };
            }
            let _ = ws.serialize_attachment(&updated_meta);
            let roster = collect_roster(&self.state);
            let update = serde_json::json!({
                "type": "room_update",
                "code": meta.code,
                "playerCount": roster.len(),
                "roster": roster
            })
            .to_string();
            for peer in self.state.get_websockets() {
                let _ = peer.send_with_str(&update);
            }
        } else if msg_type == "input" && !meta.is_host {
            val["playerIdx"] = serde_json::json!(meta.player_idx);
            let out = val.to_string();
            for peer in self.state.get_websockets() {
                if let Ok(Some(pm)) = peer.deserialize_attachment::<PeerMeta>() {
                    if pm.is_host {
                        let _ = peer.send_with_str(&out);
                    }
                }
            }
        } else if (msg_type == "state" || msg_type == "start_coop") && meta.is_host {
            for peer in self.state.get_websockets() {
                if let Ok(Some(pm)) = peer.deserialize_attachment::<PeerMeta>() {
                    if !pm.is_host {
                        let _ = peer.send_with_str(&raw);
                    }
                }
            }
        }
        Ok(())
    }

    async fn websocket_close(
        &self,
        ws: WebSocket,
        _code: usize,
        _reason: String,
        _was_clean: bool,
    ) -> Result<()> {
        if let Ok(Some(m)) = ws.deserialize_attachment::<PeerMeta>() {
            if m.is_host {
                for peer in self.state.get_websockets() {
                    if let Ok(Some(pm)) = peer.deserialize_attachment::<PeerMeta>() {
                        if !pm.is_host {
                            let _ = peer.send_with_str(r#"{"type":"room_closed"}"#);
                            let _ = peer.close(Some(1000), Some("Host left"));
                        }
                    }
                }
            } else {
                let peers = self.state.get_websockets();
                let count = peers.len().saturating_sub(1);
                let update = serde_json::json!({
                    "type": "room_update",
                    "code": m.code,
                    "playerCount": count
                })
                .to_string();
                for peer in peers {
                    if let Ok(Some(pm)) = peer.deserialize_attachment::<PeerMeta>() {
                        if pm.player_idx != m.player_idx {
                            let _ = peer.send_with_str(&update);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

fn gen_room_code() -> String {
    const CHARS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut buf = [0u8; 4];
    let now = Date::now().as_millis() as u64;
    let mut seed = now ^ 0x9E37_79B9_7F4A_7C15;
    for b in &mut buf {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        *b = CHARS[(seed as usize) % CHARS.len()];
    }
    String::from_utf8_lossy(&buf).into_owned()
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let url = req.url()?;
    let path = url.path();

    if path == "/api/create" {
        let code = gen_room_code();
        return Response::from_json(&serde_json::json!({ "code": code }));
    }

    if path == "/ws" {
        let mut code = String::new();
        for (k, v) in url.query_pairs() {
            if k == "code" {
                code = v.to_string().to_uppercase();
            }
        }
        if code.len() != 4 {
            return Response::error("Invalid room code", 400);
        }
        let ns = env.durable_object("COOP_ROOMS")?;
        let id = ns.id_from_name(&code)?;
        let stub = id.get_stub()?;
        return stub.fetch_with_request(req).await;
    }

    // Serve static assets from [assets] directory = "web"
    let assets = env.assets("ASSETS")?;
    assets.fetch_request(req).await
}
