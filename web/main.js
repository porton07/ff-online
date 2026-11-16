const canvas = document.getElementById("game-canvas");
const ctx = canvas.getContext("2d");

const ui = {
  buffBar: document.getElementById("buff-bar"),
  stageBanner: document.getElementById("stage-banner"),
  ovTitle: document.getElementById("overlay-title"),
  ovRooms: document.getElementById("overlay-rooms"),
  roomsList: document.getElementById("rooms-list"),
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
  btnLang: document.getElementById("btn-lang"),
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

const I18N = {
  en: {
    btn_mute_title: "Toggle Sound",
    helper_story: "Explore the ocean & eat your way to the top of the food chain!",
    helper_time: "See how far you can last before the Time Attack Clock runs out!",
    coop_title: "ONLINE CO-OP (2–4P)",
    create_room: "Create Room",
    room_list: "Room List",
    code_placeholder: "CODE",
    join: "Join",
    choose_stage: "Choose Stage on Map",
    controls_hint: "<b>Mouse / WASD</b> Swim • <b>Click / Space</b> Dash • <b>RMB / E</b> JD Suction • <b>Esc</b> Pause",
    open_rooms: "OPEN CO-OP ROOMS",
    refresh: "Refresh",
    back: "Back",
    lobby_title: "ONLINE CO-OP LOBBY",
    your_name: "YOUR NAME",
    name_placeholder: "Player",
    room_code: "ROOM CODE:",
    copy: "Copy",
    choose_fish: "CHOOSE YOUR FISH",
    players_in_room: "PLAYERS IN ROOM",
    leave_room: "Leave Room",
    waiting_host: "Waiting for Host to start...",
    start_arena: "▶ Start Co-Op Arena (1280×960)",
    map_hint: "Click any pearl on the map to start!",
    back_to_menu: "Back to Menu",
    paused: "PAUSED",
    resume: "Resume",
    restart_stage: "Restart Stage",
    main_menu: "Main Menu",
    stage_clear: "STAGE CLEAR!",
    next_stage: "Next Stage",
    game_over: "GAME OVER",
    you_win: "YOU WIN!",
    try_again: "Try Again",
    fish_andy_sub: "Quick Dash+",
    fish_leon_sub: "Resist + Frenzy",
    fish_eddie_sub: "Lure +8% Pts",
    fish_jd_sub: "Suction (E/RMB)",
    fish_orville_sub: "+6% Speed",
    you_tag: " (You)",
    waiting_slot: "Waiting...",
    player_default: "Player",
    loading_rooms: "Loading rooms...",
    no_rooms: "No open rooms right now. Create one!",
    failed_rooms: "Failed to load rooms.",
    host_label: "Host",
    stage_word: "Stage",
    starts_at_stage: "Starts at Stage",
    region_word: "Region",
    coop_arena: "Co-Op Arena (1280×960)",
    speed_buff: "SPEED",
    jd_suction_buff: "JD SUCTION (RMB/E)",
    score_word: "Score",
    fish_eaten_word: "Fish Eaten",
    final_score_word: "Final Score",
    high_score_word: "High Score",
    room_closed: "Host closed the Co-Op room.",
    disconnected: "Disconnected from Co-Op room.",
    cannot_create: "Cannot create Co-Op room: ",
  },
  th: {
    btn_mute_title: "เปิด/ปิดเสียง",
    helper_story: "ออกสำรวจมหาสมุทรและกินปลาเล็กเพื่อไต่ขึ้นสู่จุดสูงสุดของห่วงโซ่อาหาร!",
    helper_time: "ท้าทายตัวเองว่าจะเอาชีวิตรอดได้นานแค่ไหนก่อนหมดเวลา!",
    coop_title: "ออนไลน์ CO-OP (2–4 คน)",
    create_room: "สร้างห้อง",
    room_list: "รายการห้อง",
    code_placeholder: "รหัส",
    join: "เข้าร่วม",
    choose_stage: "เลือกด่านบนแผนที่",
    controls_hint: "<b>เมาส์ / WASD</b> ว่ายน้ำ • <b>คลิก / Space</b> พุ่ง • <b>คลิกขวา / E</b> ดูดเหยื่อ (J.D.) • <b>Esc</b> หยุดเกม",
    open_rooms: "ห้องออนไลน์ที่เปิดอยู่",
    refresh: "รีเฟรช",
    back: "กลับ",
    lobby_title: "ห้องรอออนไลน์ CO-OP",
    your_name: "ชื่อของคุณ",
    name_placeholder: "ผู้เล่น",
    room_code: "รหัสห้อง:",
    copy: "คัดลอก",
    choose_fish: "เลือกปลาของคุณ",
    players_in_room: "ผู้เล่นในห้อง",
    leave_room: "ออกจากห้อง",
    waiting_host: "รอหัวห้องกดเริ่มเกม...",
    start_arena: "▶ เริ่มเกม Co-Op (1280×960)",
    map_hint: "คลิกที่ไข่มุกบนแผนที่เพื่อเริ่มด่าน!",
    back_to_menu: "กลับเมนูหลัก",
    paused: "หยุดเกมชั่วคราว",
    resume: "เล่นต่อ",
    restart_stage: "เริ่มด่านใหม่",
    main_menu: "เมนูหลัก",
    stage_clear: "ผ่านด่าน!",
    next_stage: "ด่านถัดไป",
    game_over: "จบเกม",
    you_win: "ชนะแล้ว!",
    try_again: "ลองอีกครั้ง",
    fish_andy_sub: "พุ่งไว+",
    fish_leon_sub: "กันพิษ + Frenzy ไว",
    fish_eddie_sub: "ดึงเหยื่อ +8% แต้ม",
    fish_jd_sub: "ดูดเหยื่อ (E/คลิกขวา)",
    fish_orville_sub: "ว่ายเร็ว +6%",
    you_tag: " (คุณ)",
    waiting_slot: "รอผู้เล่น...",
    player_default: "ผู้เล่น",
    loading_rooms: "กำลังโหลดรายการห้อง...",
    no_rooms: "ยังไม่มีห้องที่เปิดอยู่ กดสร้างห้องได้เลย!",
    failed_rooms: "โหลดรายการห้องไม่สำเร็จ",
    host_label: "หัวห้อง",
    stage_word: "ด่าน",
    starts_at_stage: "เริ่มที่ด่าน",
    region_word: "โซน",
    coop_arena: "อารีน่า Co-Op (1280×960)",
    speed_buff: "เร็วขึ้น",
    jd_suction_buff: "ดูดเหยื่อ J.D. (คลิกขวา/E)",
    score_word: "คะแนน",
    fish_eaten_word: "กินปลา",
    final_score_word: "คะแนนรวม",
    high_score_word: "คะแนนสูงสุด",
    room_closed: "หัวห้องปิดห้อง Co-Op แล้ว",
    disconnected: "ขาดการเชื่อมต่อจากห้อง Co-Op",
    cannot_create: "ไม่สามารถสร้างห้อง Co-Op ได้: ",
  },
};

let currentLang = localStorage.getItem("ff_lang") || (navigator.language?.toLowerCase().startsWith("th") ? "th" : "en");
if (!I18N[currentLang]) currentLang = "en";

function t(key) {
  return (I18N[currentLang] && I18N[currentLang][key]) || I18N.en[key] || key;
}

function applyI18n() {
  document.documentElement.lang = currentLang;
  if (ui.btnLang) {
    ui.btnLang.textContent = currentLang === "th" ? "🌐 ไทย (TH)" : "🌐 English (EN)";
  }
  document.querySelectorAll("[data-i18n]").forEach((el) => {
    el.textContent = t(el.dataset.i18n);
  });
  document.querySelectorAll("[data-i18n-html]").forEach((el) => {
    el.innerHTML = t(el.dataset.i18nHtml);
  });
  document.querySelectorAll("[data-i18n-placeholder]").forEach((el) => {
    el.placeholder = t(el.dataset.i18nPlaceholder);
  });
  document.querySelectorAll("[data-i18n-title]").forEach((el) => {
    el.title = t(el.dataset.i18nTitle);
  });
  if (ui.ovLobby.classList.contains("active")) renderLobby();
  if (ui.ovRooms.classList.contains("active")) loadRoomList();
  if (lastHud) updateHud(lastHud);
}

const PLAYABLE_FISH = [
  { name: "Andy", subKey: "fish_andy_sub", img: "assets/resources/menus/map/mapangelbtn_normal.png" },
  { name: "Leon", subKey: "fish_leon_sub", img: "assets/resources/menus/map/maplionbtn_normal.png" },
  { name: "Eddie", subKey: "fish_eddie_sub", img: "assets/resources/menus/map/mapanglerbtn_normal.png" },
  { name: "J.D.", subKey: "fish_jd_sub", img: "assets/resources/menus/map/mapjdbtn_normal.png" },
  { name: "Orville", subKey: "fish_orville_sub", img: "assets/resources/menus/map/maporcabtn_normal.png" },
];

function renderLobby() {
  ui.lobbyCode.textContent = coop.code || "----";
  ui.btnLobbyStart.hidden = !coop.isHost;
  ui.lobbyWaitMsg.hidden = coop.isHost;

  ui.lobbyFishGrid.innerHTML = "";
  PLAYABLE_FISH.forEach((f, idx) => {
    const btn = document.createElement("button");
    btn.className = "fish-pick-card" + (coop.myFishIdx === idx ? " selected" : "");
    btn.innerHTML = `<img src="${f.img}" alt="${f.name}" /><span>${f.name}</span><span class="sub">${t(f.subKey)}</span>`;
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
      const entry = coop.roster[i] || { fishIdx: i % 5, name: `${t("player_default")} ${i + 1}` };
      const fi = entry.fishIdx ?? (i % 5);
      const fish = PLAYABLE_FISH[fi] || PLAYABLE_FISH[0];
      const pName = entry.name || `${t("player_default")} ${i + 1}`;
      const you = i === coop.playerIdx ? t("you_tag") : "";
      slot.className = "roster-slot filled";
      slot.innerHTML = `<img src="${fish.img}" alt="${fish.name}" /><strong>${pName}${you}</strong><span class="fish-label">P${i + 1} • ${fish.name}</span>`;
    } else {
      slot.className = "roster-slot";
      slot.innerHTML = `<strong>P${i + 1}</strong><span class="fish-label">${t("waiting_slot")}</span>`;
    }
    ui.lobbyRoster.appendChild(slot);
  }
}

let wasm = null;
let images = [];
let imageFolders = [];
const folderFallback = new Map();
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
    node.title = `${t("stage_word")} ${idx + 1}: ${entry.name} (${entry.character})`;
    node.onmouseenter = () => {
      ui.mapHoverInfo.textContent = `${t("stage_word")} ${idx + 1}: ${entry.name} • ${entry.character}`;
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
        ui.mapHoverInfo.textContent = `${ch} ${t("region_word")} • ${t("starts_at_stage")} ${firstIdx + 1}: ${manifest[firstIdx].name}`;
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

    let img = images[imgId];
    if (!img || !img.complete || img.naturalWidth === 0) {
      img = folderFallback.get(imageFolders[imgId]);
      if (!img || !img.complete || img.naturalWidth === 0) continue;
    }

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
  if (hud.buffs.speed > 0) buffs.push(`${t("speed_buff")} ${hud.buffs.speed.toFixed(0)}s`);
  if (hud.buffs.double_pts > 0) buffs.push(`2X ${hud.buffs.double_pts.toFixed(0)}s`);
  if (hud.buffs.poison > 0) buffs.push(`POISON ${hud.buffs.poison.toFixed(0)}s`);
  if (hud.buffs.stunned > 0) buffs.push(`STUNNED`);
  if (hud.player_is_jd) buffs.push(t("jd_suction_buff"));
  ui.buffBar.innerHTML = buffs.map((b) => `<span class="buff-pill">${b}</span>`).join("");

  ui.stageBanner.textContent =
    hud.state === "title"
      ? ""
      : coop.active
      ? `${t("coop_arena")} • ${coop.playerCount}P`
      : `${hud.character} • ${t("stage_word")} ${hud.stage_idx + 1}/${hud.stage_count}: ${hud.stage_name}`;

  ui.ovTitle.classList.toggle(
    "active",
    hud.state === "title" &&
      !ui.ovMap.classList.contains("active") &&
      !ui.ovLobby.classList.contains("active") &&
      !ui.ovRooms.classList.contains("active")
  );
  ui.ovPause.classList.toggle("active", hud.state === "paused");
  ui.ovClear.classList.toggle("active", hud.state === "stage_clear");
  ui.ovGameOver.classList.toggle(
    "active",
    hud.state === "game_over" || hud.state === "game_won"
  );

  if (hud.state === "stage_clear") {
    ui.clearSummary.textContent = `${t("score_word")}: ${hud.score.toLocaleString()} • ${t("fish_eaten_word")}: ${hud.fish_eaten}`;
  } else if (hud.state === "game_over" || hud.state === "game_won") {
    ui.goTitle.textContent = hud.state === "game_won" ? t("you_win") : t("game_over");
    ui.goSummary.textContent = `${t("final_score_word")}: ${hud.score.toLocaleString()} • ${t("high_score_word")}: ${hud.high_score.toLocaleString()}`;
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
      alert(t("room_closed"));
      location.reload();
    } else if (msg.type === "error") {
      alert(msg.message);
    }
  };
  ws.onclose = () => {
    if (coop.active && !coop.isHost) {
      ui.coopStatus.textContent = t("disconnected");
    }
  };
}

async function loadRoomList() {
  ui.roomsList.innerHTML = `<div class="room-empty">${t("loading_rooms")}</div>`;
  try {
    const r = await fetch("/api/rooms");
    const list = await r.json();
    if (!Array.isArray(list) || list.length === 0) {
      ui.roomsList.innerHTML = `<div class="room-empty">${t("no_rooms")}</div>`;
      return;
    }
    ui.roomsList.innerHTML = "";
    list.forEach((rm) => {
      const row = document.createElement("div");
      row.className = "room-item";
      row.innerHTML = `<div class="room-item-info"><strong>${rm.code}</strong><span>${t("host_label")}: ${rm.hostName} • ${rm.count}/4</span></div><button class="btn-gold-sm">${t("join")}</button>`;
      row.querySelector("button").onclick = () => {
        ui.ovRooms.classList.remove("active");
        connectCoopWs(`/ws?code=${encodeURIComponent(rm.code)}&role=join`);
      };
      ui.roomsList.appendChild(row);
    });
  } catch {
    ui.roomsList.innerHTML = `<div class="room-empty">${t("failed_rooms")}</div>`;
  }
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
      alert(t("cannot_create") + err.message);
    }
  };
  document.getElementById("btn-coop-rooms").onclick = () => {
    ui.ovTitle.classList.remove("active");
    ui.ovRooms.classList.add("active");
    loadRoomList();
  };
  document.getElementById("btn-rooms-refresh").onclick = () => loadRoomList();
  document.getElementById("btn-rooms-close").onclick = () => {
    ui.ovRooms.classList.remove("active");
    ui.ovTitle.classList.add("active");
  };
  document.getElementById("btn-coop-join").onclick = () => {
    const code = ui.coopCodeInput.value.trim().toUpperCase();
    if (!code) return;
    connectCoopWs(`/ws?code=${encodeURIComponent(code)}&role=join`);
  };
  document.getElementById("btn-coop-start").onclick = () => startStage(32, false, true);
  document.getElementById("btn-lobby-start").onclick = () => startStage(32, false, true);
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

  ui.btnLang.onclick = () => {
    currentLang = currentLang === "th" ? "en" : "th";
    localStorage.setItem("ff_lang", currentLang);
    applyI18n();
  };

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
  applyI18n();
  const resp = await fetch("feeding_frenzy_wasm.wasm");
  const bytes = await resp.arrayBuffer();
  const { instance } = await WebAssembly.instantiate(bytes, {});
  wasm = instance.exports;

  wasm.init((Date.now() & 0xffff_ffff) >>> 0);
  const initData = JSON.parse(readStr(wasm.get_init_ptr(), wasm.get_init_len()));
  manifest = initData.manifest;
  imageFolders = initData.images.map((src) => {
    const slash = src.lastIndexOf("/");
    return slash >= 0 ? src.slice(0, slash) : src;
  });
  images = initData.images.map((src, idx) => {
    const im = new Image();
    const folder = imageFolders[idx];
    im.onload = () => {
      if (!folderFallback.has(folder) || src.includes("swim")) {
        folderFallback.set(folder, im);
      }
    };
    im.src = src;
    return im;
  });

  buildStageGrid();
  requestAnimationFrame(loop);
}

boot();
