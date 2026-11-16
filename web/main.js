const canvas = document.getElementById("game-canvas");
const ctx = canvas.getContext("2d");

const ui = {
  buffBar: document.getElementById("buff-bar"),
  stageBanner: document.getElementById("stage-banner"),
  ovTitle: document.getElementById("overlay-title"),
  ovLobby: document.getElementById("overlay-lobby"),
  lobbyCode: document.getElementById("lobby-code"),
  lobbyNameInput: document.getElementById("lobby-name-input"),
  lobbyFishGrid: document.getElementById("lobby-fish-grid"),
  lobbyRoster: document.getElementById("lobby-roster"),
  btnLobbyMap: document.getElementById("btn-lobby-map"),
  btnLobbyStart: document.getElementById("btn-lobby-start"),
  lobbyWaitMsg: document.getElementById("lobby-wait-msg"),
  ovMap: document.getElementById("overlay-map"),
  mapNodes: document.getElementById("map-nodes"),
  mapTrail: document.getElementById("map-trail"),
  mapHoverInfo: document.getElementById("map-hover-info"),
  ovPause: document.getElementById("overlay-pause"),
  ovClear: document.getElementById("overlay-clear"),
  ovGameOver: document.getElementById("overlay-gameover"),
  clearSummary: document.getElementById("clear-summary"),
  goTitle: document.getElementById("gameover-title"),
  goSummary: document.getElementById("gameover-summary"),
  btnMute: document.getElementById("btn-mute"),
  coopStatus: document.getElementById("coop-status"),
  coopLobby: document.getElementById("coop-lobby"),
  btnCoopStart: document.getElementById("btn-coop-start"),
  coopCodeInput: document.getElementById("coop-code-input"),
};

let selectedChar = "Andy";

let coop = {
  ws: null,
  active: false,
  isHost: false,
  playerIdx: 0,
  playerCount: 1,
  code: "",
  netTick: 0,
  myFishIdx: 0,
  myName: localStorage.getItem("ff_name") || "",
  roster: [{ playerIdx: 0, fishIdx: 0, name: "Player 1" }],
};

const PLAYABLE_FISH = [
  { name: "Andy", sub: "Angelfish", img: "assets/resources/menus/map/mapangelbtn_normal.png" },
  { name: "Leon", sub: "Lionfish", img: "assets/resources/menus/map/maplionbtn_normal.png" },
  { name: "Eddie", sub: "Angler", img: "assets/resources/menus/map/mapanglerbtn_normal.png" },
  { name: "J.D.", sub: "Suction", img: "assets/resources/menus/map/mapjdbtn_normal.png" },
  { name: "Orville", sub: "Orca", img: "assets/resources/menus/map/maporcabtn_normal.png" },
];

function renderLobby() {
  ui.lobbyCode.textContent = coop.code || "----";
  ui.btnLobbyMap.hidden = !coop.isHost;
  ui.btnLobbyStart.hidden = !coop.isHost;
  ui.lobbyWaitMsg.hidden = coop.isHost;

  ui.lobbyFishGrid.innerHTML = "";
  PLAYABLE_FISH.forEach((f, idx) => {
    const btn = document.createElement("button");
    btn.className = "fish-pick-card" + (coop.myFishIdx === idx ? " selected" : "");
    btn.innerHTML = `<img src="${f.img}" alt="${f.name}" /><span>${f.name}</span><span class="sub">${f.sub}</span>`;
    btn.onclick = () => {
      coop.myFishIdx = idx;
      if (coop.isHost) {
        wasm.set_player_fish(0, idx);
      }
      if (coop.ws && coop.ws.readyState === 1) {
        coop.ws.send(JSON.stringify({ type: "pick_fish", fishIdx: idx }));
      }
      renderLobby();
    };
    ui.lobbyFishGrid.appendChild(btn);
  });

  ui.lobbyRoster.innerHTML = "";
  for (let i = 0; i < 4; i++) {
    const slot = document.createElement("div");
    if (i < coop.playerCount) {
      const entry = coop.roster[i] || { fishIdx: i % 5, name: `Player ${i + 1}` };
      const fi = entry.fishIdx ?? (i % 5);
      const fish = PLAYABLE_FISH[fi] || PLAYABLE_FISH[0];
      const pName = entry.name || `Player ${i + 1}`;
      const you = i === coop.playerIdx ? " (You)" : "";
      slot.className = "roster-slot filled";
      slot.innerHTML = `<img src="${fish.img}" alt="${fish.name}" /><strong>${pName}${you}</strong><span class="fish-label">P${i + 1} • ${fish.name}</span>`;
    } else {
      slot.className = "roster-slot";
      slot.innerHTML = `<strong>P${i + 1}</strong><span class="fish-label">Waiting...</span>`;
    }
    ui.lobbyRoster.appendChild(slot);
  }
}

let wasm = null;
let images = [];
let manifest = [];
let muted = false;
let currentMusicUrl = "";
let bgmAudio = null;
const sfxCache = new Map();

const input = {
  mouseX: 400,
  mouseY: 300,
  mouseActive: false,
  keyMask: 0,
  dashQueued: false,
  specialQueued: false,
};

let lastHud = null;
let lastTime = performance.now();

const hudSprites = {};
function loadHudSprites() {
  const files = {
    bg: "assets/resources/hud/hudbg.jpg",
    bubble: "assets/resources/hud/hudbubble.png",
    danger: "assets/resources/hud/dangersign.png",
    timer: "assets/resources/hud/timerbg.png",
    marker: "assets/resources/hud/growmetermarker.png",
  };
  for (const [k, src] of Object.entries(files)) {
    const im = new Image();
    im.src = src;
    hudSprites[k] = im;
  }
}
const iconImgCache = new Map();
function getIconImg(src) {
  let im = iconImgCache.get(src);
  if (!im) {
    im = new Image();
    im.src = src;
    iconImgCache.set(src, im);
  }
  return im;
}

function playSfx(name) {
  if (muted) return;
  const url = `assets/resources/sounds/${name}`;
  let base = sfxCache.get(url);
  if (!base) {
    base = new Audio(url);
    sfxCache.set(url, base);
  }
  const inst = base.cloneNode();
  inst.volume = 0.55;
  inst.play().catch(() => {});
}

function updateMusic(url) {
  if (!url || url === currentMusicUrl) return;
  currentMusicUrl = url;
  if (bgmAudio) {
    bgmAudio.pause();
    bgmAudio = null;
  }
  if (muted) return;
  bgmAudio = new Audio(url);
  bgmAudio.loop = true;
  bgmAudio.volume = 0.35;
  bgmAudio.play().catch(() => {});
}

function readStr(ptr, len) {
  const bytes = new Uint8Array(wasm.memory.buffer, ptr, len);
  return new TextDecoder().decode(bytes);
}

function openStoryMap() {
  ui.ovTitle.classList.remove("active");
  ui.ovMap.classList.add("active");
}

function closeStoryMap() {
  ui.ovMap.classList.remove("active");
  ui.ovTitle.classList.add("active");
}

function buildStageGrid() {
  ui.mapNodes.innerHTML = "";
  const pts = [];
  manifest.forEach((entry, idx) => {
    const [mx, my] = entry.mapPos || [320, 240];
    pts.push(`${mx},${my}`);
    const node = document.createElement("button");
    node.className = `map-node char-${entry.character}`;
    node.style.left = `${(mx / 640) * 100}%`;
    node.style.top = `${(my / 480) * 100}%`;
    node.textContent = String(idx + 1);
    node.title = `Stage ${idx + 1}: ${entry.name} (${entry.character})`;
    node.onmouseenter = () => {
      ui.mapHoverInfo.textContent = `Stage ${idx + 1}: ${entry.name} • ${entry.character}`;
    };
    node.onclick = () => {
      ui.ovMap.classList.remove("active");
      startStage(idx, false, true);
    };
    ui.mapNodes.appendChild(node);
  });

  ui.mapTrail.innerHTML = `<polyline points="${pts.join(" ")}" fill="none" stroke="rgba(255,255,255,0.55)" stroke-width="2" stroke-dasharray="5,4" />`;

  document.querySelectorAll(".orig-region-btn").forEach((btn) => {
    const ch = btn.dataset.char;
    const firstIdx = manifest.findIndex((m) => m.character === ch);
    if (firstIdx >= 0) {
      btn.onmouseenter = () => {
        ui.mapHoverInfo.textContent = `${ch} Region • Starts at Stage ${firstIdx + 1}: ${manifest[firstIdx].name}`;
      };
      btn.onclick = () => {
        ui.ovMap.classList.remove("active");
        startStage(firstIdx, false, true);
      };
    }
  });
}

function startStage(idx, timedMode, resetRun) {
  ui.ovLobby.classList.remove("active");
  if (coop.active && coop.isHost) {
    coop.roster.forEach((entry, pIdx) => {
      const fi = typeof entry === "number" ? entry : (entry.fishIdx ?? 0);
      wasm.set_player_fish(pIdx, fi);
    });
    wasm.start_coop_stage(idx, timedMode ? 1 : 0, resetRun ? 1 : 0, coop.playerCount);
    if (coop.ws && coop.ws.readyState === 1) {
      coop.ws.send(JSON.stringify({ type: "start_coop", stageIdx: idx }));
    }
  } else {
    wasm.start_stage(idx, timedMode ? 1 : 0, resetRun ? 1 : 0);
  }
  if (bgmAudio && bgmAudio.paused && !muted) {
    bgmAudio.play().catch(() => {});
  }
}

function renderFrame(hud, frameArray) {
  const w = hud.stage_width || 800;
  const h = hud.stage_height || 600;
  if (canvas.width !== w || canvas.height !== h) {
    canvas.width = w;
    canvas.height = h;
  }

  ctx.clearRect(0, 0, w, h);

  // Subtle aquatic backdrop fallback
  const grad = ctx.createLinearGradient(0, 0, 0, h);
  grad.addColorStop(0, "#0b5c8a");
  grad.addColorStop(1, "#032138");
  ctx.fillStyle = grad;
  ctx.fillRect(0, 0, w, h);

  let cmds = frameArray;
  if (!cmds) {
    const ptr = wasm.get_frame_ptr();
    const len = wasm.get_frame_len();
    cmds = new Float32Array(wasm.memory.buffer, ptr, len);
  }

  for (let i = 0; i < cmds.length; i += 9) {
    const imgId = cmds[i] | 0;
    const x = cmds[i + 1];
    const y = cmds[i + 2];
    const dw = cmds[i + 3];
    const dh = cmds[i + 4];
    const flipX = cmds[i + 5];
    const rot = cmds[i + 6];
    const alpha = cmds[i + 7];
    const hue = cmds[i + 8];

    const img = images[imgId];
    if (!img || !img.complete || img.naturalWidth === 0) continue;

    ctx.save();
    ctx.globalAlpha = alpha;
    if (Math.abs(hue) > 1.0) {
      ctx.filter = `hue-rotate(${hue}deg)`;
    }
    ctx.translate(x, y);
    if (rot !== 0) ctx.rotate(rot);
    if (flipX < 0) ctx.scale(-1, 1);
    ctx.drawImage(img, -dw * 0.5, -dh * 0.5, dw, dh);
    ctx.restore();
  }

  // Particles
  for (const pt of hud.particles) {
    const t = pt.life / Math.max(pt.max_life, 0.001);
    const alpha = Math.max(0, 1 - t);
    ctx.save();
    ctx.globalAlpha = alpha * 0.85;
    if (pt.kind === 0) {
      ctx.strokeStyle = "#b3ecff";
      ctx.fillStyle = "rgba(180, 240, 255, 0.25)";
      ctx.lineWidth = 1.4;
      ctx.beginPath();
      ctx.arc(pt.x, pt.y, pt.r, 0, Math.PI * 2);
      ctx.fill();
      ctx.stroke();
    } else if (pt.kind === 1) {
      ctx.fillStyle = "#ffe066";
      ctx.beginPath();
      ctx.arc(pt.x, pt.y, pt.r * (1 - t * 0.5), 0, Math.PI * 2);
      ctx.fill();
    } else {
      ctx.fillStyle = "#ff8a80";
      ctx.beginPath();
      ctx.arc(pt.x, pt.y, pt.r * 0.8, 0, Math.PI * 2);
      ctx.fill();
    }
    ctx.restore();
  }

  // Predator edge warnings (dangersign.png)
  if (hud.warnings) {
    const dImg = hudSprites.danger;
    for (const w of hud.warnings) {
      ctx.save();
      if (dImg && dImg.complete && dImg.naturalWidth > 0) {
        ctx.drawImage(dImg, w.x - 22, w.y - 20, 45, 40);
      } else {
        ctx.fillStyle = "#ff1744";
        ctx.beginPath();
        ctx.arc(w.x, w.y, 16, 0, Math.PI * 2);
        ctx.fill();
      }
      ctx.restore();
    }
  }

  // Authentic In-Canvas HUD (scaled proportionally to canvas width w)
  if (hud.state !== "title") {
    ctx.save();
    ctx.scale(w / 800, 1);
    const hBg = hudSprites.bg;
    if (hBg && hBg.complete && hBg.naturalWidth > 0) {
      ctx.drawImage(hBg, 0, 0, 800, 105);
    }

    // 4 MENU Fish Bubbles
    const bImg = hudSprites.bubble;
    const slotXs = [121, 215, 309, 415];
    for (let i = 0; i < 4; i++) {
      const sx = slotXs[i];
      const sy = 35;
      if (bImg && bImg.complete && bImg.naturalWidth > 0) {
        ctx.drawImage(bImg, sx - 30, sy - 25, 60, 50);
      }
      const icon = hud.edible_icons && hud.edible_icons[i];
      if (icon) {
        const im = getIconImg(icon.image);
        if (im.complete && im.naturalWidth > 0) {
          ctx.save();
          if (Math.abs(icon.hue) > 1) ctx.filter = `hue-rotate(${icon.hue}deg)`;
          const ratio = Math.min(40 / im.naturalWidth, 30 / im.naturalHeight);
          const dw = im.naturalWidth * ratio;
          const dh = im.naturalHeight * ratio;
          ctx.drawImage(im, sx - dw * 0.5, sy - dh * 0.5, dw, dh);
          ctx.restore();
        }
      }
    }

    // Growth Bar (aligned to hudbg.jpg: x=98..435, y=75..85)
    const gx = 98;
    const gy = 75;
    const gw = 337;
    const gh = 11;
    ctx.fillStyle = "rgba(0, 12, 28, 0.85)";
    ctx.fillRect(gx, gy, gw, gh);
    const gGrad = ctx.createLinearGradient(gx, 0, gx + gw, 0);
    gGrad.addColorStop(0, "#00e676");
    gGrad.addColorStop(1, "#aeea00");
    ctx.fillStyle = gGrad;
    ctx.fillRect(gx + 1, gy + 1, Math.max(0, (gw - 2) * hud.growth_progress), gh - 2);
    if (hud.tier_marks) {
      const mImg = hudSprites.marker;
      for (const m of hud.tier_marks) {
        const mx = gx + gw * m;
        if (mImg && mImg.complete && mImg.naturalWidth > 0) {
          ctx.drawImage(mImg, mx - 8, gy - 2, 16, 13);
        } else {
          ctx.fillStyle = "#fff";
          ctx.fillRect(mx - 1, gy, 2, gh);
        }
      }
    }

    // Frenzy / Ability Bar (aligned to hudbg.jpg: x=542..668, y=75..86)
    const fx = 542;
    const fy = 75;
    const fw = 126;
    const fh = 11;
    ctx.fillStyle = "rgba(0, 12, 28, 0.85)";
    ctx.fillRect(fx, fy, fw, fh);
    const fGrad = ctx.createLinearGradient(fx, 0, fx + fw, 0);
    fGrad.addColorStop(0, "#ffea00");
    fGrad.addColorStop(0.6, "#ff6d00");
    fGrad.addColorStop(1, "#ff1744");
    ctx.fillStyle = fGrad;
    ctx.fillRect(fx + 1, fy + 1, Math.max(0, (fw - 2) * hud.frenzy), fh - 2);

    // Score & Lives & Multiplier (aligned to hudbg.jpg boxes)
    ctx.save();
    ctx.font = '900 26px "Segoe UI", sans-serif';
    ctx.textAlign = "right";
    ctx.strokeStyle = "rgba(0,0,0,0.85)";
    ctx.lineWidth = 4;
    const scoreStr = hud.score.toLocaleString();
    ctx.strokeText(scoreStr, 692, 42);
    ctx.fillStyle = "#ffffff";
    ctx.fillText(scoreStr, 692, 42);

    ctx.font = '900 20px "Segoe UI", sans-serif';
    ctx.textAlign = "center";
    const lifeStr = `${Math.max(0, hud.lives)}`;
    ctx.strokeText(lifeStr, 760, 88);
    ctx.fillStyle = "#ffe066";
    ctx.fillText(lifeStr, 760, 88);

    if (hud.frenzy_mult > 1) {
      ctx.textAlign = "center";
      ctx.font = '900 22px "Segoe UI", sans-serif';
      ctx.strokeText(`${hud.frenzy_mult}X`, 750, 38);
      ctx.fillStyle = "#ffea00";
      ctx.fillText(`${hud.frenzy_mult}X`, 750, 38);
    }
    ctx.restore();
    ctx.restore();

    // Timer at bottom center for Bonus Stages / Time Attack
    if (hud.timed_mode) {
      const tImg = hudSprites.timer;
      const tx = w * 0.5;
      const ty = h - 42;
      if (tImg && tImg.complete && tImg.naturalWidth > 0) {
        ctx.drawImage(tImg, tx - 80, ty - 26, 160, 52);
      }
      const sec = Math.ceil(hud.time_remaining);
      const m = Math.floor(sec / 60);
      const s = String(sec % 60).padStart(2, "0");
      ctx.save();
      ctx.font = '900 26px "Segoe UI", sans-serif';
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      ctx.strokeStyle = "rgba(0,0,0,0.85)";
      ctx.lineWidth = 4;
      ctx.strokeText(`${m}:${s}`, tx, ty);
      ctx.fillStyle = sec <= 10 ? "#ff5252" : "#ffffff";
      ctx.fillText(`${m}:${s}`, tx, ty);
      ctx.restore();
    }
  }

  // Co-Op Player Tags (Nickname or P1..P4)
  if (hud.player_tags) {
    hud.player_tags.forEach((t, idx) => {
      if (!t.alive) return;
      const customName = coop.roster[idx]?.name || t.label;
      ctx.save();
      ctx.font = '900 13px "Segoe UI", sans-serif';
      ctx.textAlign = "center";
      ctx.lineWidth = 3;
      ctx.strokeStyle = "rgba(0,0,0,0.85)";
      ctx.strokeText(customName, t.x, t.y);
      ctx.fillStyle = t.color;
      ctx.fillText(customName, t.x, t.y);
      ctx.restore();
    });
  }

  // Floating popups
  for (const pp of hud.popups) {
    const t = pp.life / Math.max(pp.max_life, 0.001);
    ctx.save();
    ctx.globalAlpha = Math.max(0, 1 - t * t);
    ctx.font = `900 ${Math.round(16 * pp.scale)}px "Segoe UI", sans-serif`;
    ctx.textAlign = "center";
    ctx.lineWidth = 3;
    ctx.strokeStyle = "rgba(0,0,0,0.75)";
    ctx.strokeText(pp.text, pp.x, pp.y);
    ctx.fillStyle = pp.color;
    ctx.fillText(pp.text, pp.x, pp.y);
    ctx.restore();
  }
}

function updateUi(hud) {
  const buffs = [];
  if (hud.buffs.shield) buffs.push("SHIELD");
  if (hud.buffs.speed > 0) buffs.push(`SPEED ${hud.buffs.speed.toFixed(0)}s`);
  if (hud.buffs.double_pts > 0) buffs.push(`2X ${hud.buffs.double_pts.toFixed(0)}s`);
  if (hud.buffs.poison > 0) buffs.push(`POISON ${hud.buffs.poison.toFixed(0)}s`);
  if (hud.buffs.stunned > 0) buffs.push(`STUNNED`);
  if (hud.player_is_jd) buffs.push(`JD SUCTION (RMB/E)`);
  ui.buffBar.innerHTML = buffs.map((b) => `<span class="buff-pill">${b}</span>`).join("");

  ui.stageBanner.textContent =
    hud.state === "title"
      ? ""
      : `${hud.character} • Stage ${hud.stage_idx + 1}/${hud.stage_count}: ${hud.stage_name}`;

  ui.ovTitle.classList.toggle(
    "active",
    hud.state === "title" &&
      !ui.ovMap.classList.contains("active") &&
      !ui.ovLobby.classList.contains("active")
  );
  ui.ovPause.classList.toggle("active", hud.state === "paused");
  ui.ovClear.classList.toggle("active", hud.state === "stage_clear");
  ui.ovGameOver.classList.toggle(
    "active",
    hud.state === "game_over" || hud.state === "game_won"
  );

  if (hud.state === "stage_clear") {
    ui.clearSummary.textContent = `Score: ${hud.score.toLocaleString()} • Fish Eaten: ${hud.fish_eaten}`;
  } else if (hud.state === "game_over" || hud.state === "game_won") {
    ui.goTitle.textContent = hud.state === "game_won" ? "YOU WIN!" : "GAME OVER";
    ui.goSummary.textContent = `Final Score: ${hud.score.toLocaleString()} • High Score: ${hud.high_score.toLocaleString()}`;
  }

  for (const s of hud.sounds) {
    playSfx(s);
  }
  updateMusic(hud.music);
}

function loop(now) {
  const dt = Math.min((now - lastTime) / 1000, 0.05);
  lastTime = now;

  if (coop.active && !coop.isHost) {
    // Co-Op Client: send local input to Host over WebSocket
    coop.netTick++;
    if (coop.ws && coop.ws.readyState === 1 && (coop.netTick % 2 === 0 || input.dashQueued || input.specialQueued)) {
      coop.ws.send(
        JSON.stringify({
          type: "input",
          mx: Math.round(input.mouseX),
          my: Math.round(input.mouseY),
          ma: input.mouseActive ? 1 : 0,
          km: input.keyMask,
          dash: input.dashQueued ? 1 : 0,
          spec: input.specialQueued ? 1 : 0,
        })
      );
      input.dashQueued = false;
      input.specialQueued = false;
    }
    requestAnimationFrame(loop);
    return;
  }

  wasm.set_player_input(
    0,
    input.mouseX,
    input.mouseY,
    input.mouseActive ? 1 : 0,
    input.keyMask,
    input.dashQueued ? 1 : 0,
    input.specialQueued ? 1 : 0
  );
  input.dashQueued = false;
  input.specialQueued = false;

  wasm.tick(dt);

  const hud = JSON.parse(readStr(wasm.get_hud_ptr(), wasm.get_hud_len()));
  lastHud = hud;
  renderFrame(hud);
  updateUi(hud);

  if (coop.active && coop.isHost && coop.ws && coop.ws.readyState === 1) {
    coop.netTick++;
    if (coop.netTick % 2 === 0 || hud.sounds.length > 0 || hud.state !== "playing") {
      const ptr = wasm.get_frame_ptr();
      const len = wasm.get_frame_len();
      const cmds = Array.from(new Float32Array(wasm.memory.buffer, ptr, len));
      coop.ws.send(JSON.stringify({ type: "state", frame: cmds, hud }));
    }
  }

  requestAnimationFrame(loop);
}

function connectCoopWs(wsPath, onOpen) {
  const proto = location.protocol === "https:" ? "wss:" : "ws:";
  const ws = new WebSocket(`${proto}//${location.host}${wsPath}`);
  coop.ws = ws;
  if (onOpen) ws.onopen = onOpen;
  ws.onmessage = (ev) => {
    const msg = JSON.parse(ev.data);
    if (msg.type === "created") {
      coop.active = true;
      coop.isHost = true;
      coop.playerIdx = 0;
      coop.code = msg.code;
      coop.myFishIdx = 0;
      coop.roster = [{ playerIdx: 0, fishIdx: 0, name: coop.myName || "Player 1" }];
      ui.lobbyNameInput.value = coop.myName;
      if (coop.myName) ws.send(JSON.stringify({ type: "set_name", name: coop.myName }));
      ui.ovTitle.classList.remove("active");
      ui.ovLobby.classList.add("active");
      renderLobby();
    } else if (msg.type === "joined") {
      coop.active = true;
      coop.isHost = false;
      coop.playerIdx = msg.playerIdx;
      coop.code = msg.code;
      coop.myFishIdx = msg.playerIdx % 5;
      ui.lobbyNameInput.value = coop.myName;
      if (coop.myName) ws.send(JSON.stringify({ type: "set_name", name: coop.myName }));
      ui.ovTitle.classList.remove("active");
      ui.ovLobby.classList.add("active");
      renderLobby();
    } else if (msg.type === "room_update") {
      coop.playerCount = msg.playerCount;
      if (Array.isArray(msg.roster)) {
        coop.roster = msg.roster;
        if (coop.isHost) {
          coop.roster.forEach((entry, pIdx) => {
            const fi = typeof entry === "number" ? entry : (entry.fishIdx ?? 0);
            wasm.set_player_fish(pIdx, fi);
          });
        }
      }
      renderLobby();
    } else if (msg.type === "start_coop" && !coop.isHost) {
      ui.ovLobby.classList.remove("active");
      ui.ovMap.classList.remove("active");
    } else if (msg.type === "input" && coop.isHost) {
      wasm.set_player_input(
        msg.playerIdx,
        msg.mx,
        msg.my,
        msg.ma,
        msg.km,
        msg.dash,
        msg.spec
      );
    } else if (msg.type === "state" && !coop.isHost) {
      lastHud = msg.hud;
      renderFrame(msg.hud, msg.frame);
      updateUi(msg.hud);
    } else if (msg.type === "room_closed") {
      alert("Host closed the Co-Op room.");
      location.reload();
    } else if (msg.type === "error") {
      alert(msg.message);
    }
  };
  ws.onclose = () => {
    if (coop.active && !coop.isHost) {
      ui.coopStatus.textContent = "Disconnected from Co-Op room.";
    }
  };
}

function setupInput() {
  canvas.addEventListener("mousemove", (e) => {
    const rect = canvas.getBoundingClientRect();
    input.mouseX = ((e.clientX - rect.left) / rect.width) * canvas.width;
    input.mouseY = ((e.clientY - rect.top) / rect.height) * canvas.height;
    input.mouseActive = true;
  });

  canvas.addEventListener("mousedown", (e) => {
    if (e.button === 0) input.dashQueued = true;
    if (e.button === 2) input.specialQueued = true;
  });

  canvas.addEventListener("contextmenu", (e) => e.preventDefault());

  const keyBits = {
    ArrowUp: 1,
    KeyW: 1,
    ArrowDown: 2,
    KeyS: 2,
    ArrowLeft: 4,
    KeyA: 4,
    ArrowRight: 8,
    KeyD: 8,
  };

  window.addEventListener("keydown", (e) => {
    if (e.code in keyBits) {
      input.keyMask |= keyBits[e.code];
      input.mouseActive = false;
    } else if (e.code === "Space" || e.code === "ShiftLeft" || e.code === "ShiftRight") {
      input.dashQueued = true;
    } else if (e.code === "KeyE") {
      input.specialQueued = true;
    } else if (e.code === "Escape" || e.code === "KeyP") {
      if (wasm) wasm.toggle_pause();
    }
  });

  window.addEventListener("keyup", (e) => {
    if (e.code in keyBits) {
      input.keyMask &= ~keyBits[e.code];
    }
  });

  document.getElementById("btn-play-story").onclick = () => openStoryMap();
  document.getElementById("btn-map-back").onclick = () => closeStoryMap();
  document.getElementById("btn-play-timed").onclick = () => startStage(0, true, true);
  document.getElementById("btn-coop-create").onclick = async () => {
    try {
      const r = await fetch("/api/create");
      if (!r.ok) throw new Error(`HTTP ${r.status}`);
      const { code } = await r.json();
      connectCoopWs(`/ws?code=${encodeURIComponent(code)}&role=host`);
    } catch (err) {
      alert("Cannot create Co-Op room: " + err.message);
    }
  };
  document.getElementById("btn-coop-join").onclick = () => {
    const code = ui.coopCodeInput.value.trim().toUpperCase();
    if (!code) return;
    connectCoopWs(`/ws?code=${encodeURIComponent(code)}&role=join`);
  };
  document.getElementById("btn-coop-start").onclick = () => openStoryMap();
  document.getElementById("btn-lobby-map").onclick = () => {
    ui.ovLobby.classList.remove("active");
    openStoryMap();
  };
  document.getElementById("btn-lobby-start").onclick = () => startStage(0, false, true);
  document.getElementById("btn-lobby-leave").onclick = () => location.reload();
  ui.lobbyNameInput.addEventListener("input", () => {
    coop.myName = ui.lobbyNameInput.value.trim().slice(0, 12);
    localStorage.setItem("ff_name", coop.myName);
    if (coop.ws && coop.ws.readyState === 1) {
      coop.ws.send(JSON.stringify({ type: "set_name", name: coop.myName }));
    }
  });
  document.getElementById("btn-copy-code").onclick = () => {
    navigator.clipboard?.writeText(coop.code);
  };
  document.getElementById("btn-resume").onclick = () => wasm.toggle_pause();
  document.getElementById("btn-restart-stage").onclick = () => {
    if (lastHud) startStage(lastHud.stage_idx, lastHud.timed_mode, false);
  };
  document.getElementById("btn-next-stage").onclick = () => {
    if (lastHud) startStage(lastHud.stage_idx + 1, lastHud.timed_mode, false);
  };
  document.getElementById("btn-retry").onclick = () => {
    if (lastHud) startStage(lastHud.stage_idx, lastHud.timed_mode, true);
  };
  const backToTitle = () => location.reload();
  document.getElementById("btn-quit-menu").onclick = backToTitle;
  document.getElementById("btn-go-menu").onclick = backToTitle;
  document.getElementById("btn-menu").onclick = backToTitle;
  document.getElementById("btn-pause").onclick = () => wasm && wasm.toggle_pause();

  ui.btnMute.onclick = () => {
    muted = !muted;
    document.getElementById("icon-sound-on").hidden = muted;
    document.getElementById("icon-sound-off").hidden = !muted;
    if (bgmAudio) {
      if (muted) bgmAudio.pause();
      else bgmAudio.play().catch(() => {});
    }
  };
}

async function boot() {
  loadHudSprites();
  setupInput();
  const resp = await fetch("feeding_frenzy_wasm.wasm");
  const bytes = await resp.arrayBuffer();
  const { instance } = await WebAssembly.instantiate(bytes, {});
  wasm = instance.exports;

  wasm.init((Date.now() & 0xffff_ffff) >>> 0);
  const initData = JSON.parse(readStr(wasm.get_init_ptr(), wasm.get_init_len()));
  manifest = initData.manifest;
  images = initData.images.map((src) => {
    const im = new Image();
    im.src = src;
    return im;
  });

  buildStageGrid();
  requestAnimationFrame(loop);
}

boot();
