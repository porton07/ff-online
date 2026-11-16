import json
import os
import re
import shutil
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
RAW = ROOT / "assets_raw"
GAME_RES = ROOT.parent / "resources"
WEB_ASSETS = ROOT / "web" / "assets"
OUT_JSON = ROOT / "crates" / "game" / "src" / "game_data.json"

ATTR_RE = re.compile(r'([A-Za-z_][A-Za-z0-9_:-]*)\s*=\s*"([^"]*)"')
TAG_RE = re.compile(r'<!--.*?-->|<([/?!]?[A-Za-z_][A-Za-z0-9_:-]*)([^>]*)>', re.DOTALL)


def parse_lenient_xml(text: str):
    """Parse XML leniently (allowing duplicate attributes and unescaped quirks)."""
    stack = [{"tag": "__root__", "attrs": {}, "children": []}]
    for m in TAG_RE.finditer(text):
        tag = m.group(1)
        if tag is None:
            continue  # comment
        if tag.startswith("?") or tag.startswith("!"):
            continue
        if tag.startswith("/"):
            if len(stack) > 1:
                stack.pop()
            continue
        raw_attrs = m.group(2) or ""
        self_closing = raw_attrs.rstrip().endswith("/")
        attrs = {}
        for am in ATTR_RE.finditer(raw_attrs):
            attrs[am.group(1)] = am.group(2)
        node = {"tag": tag, "attrs": attrs, "children": []}
        stack[-1]["children"].append(node)
        if not self_closing:
            stack.append(node)
    return stack[0]["children"][0] if stack[0]["children"] else None


def find_raw_ci(rel_path: str) -> Path | None:
    rel = rel_path.replace("\\", "/").lstrip("/")
    p = RAW / rel
    if p.exists():
        return p
    # Case-insensitive fallback
    parts = rel.split("/")
    cur = RAW
    for part in parts:
        if not cur.is_dir():
            return None
        low = part.lower()
        match = None
        for child in cur.iterdir():
            if child.name.lower() == low:
                match = child
                break
        if match is None:
            return None
        cur = match
    return cur


def parse_range(val: str | None, default=(0.0, 0.0)):
    if not val:
        return [default[0], default[1]]
    s = val.strip()
    if s.startswith("(") and s.endswith(")"):
        parts = [p.strip() for p in s[1:-1].split(",")]
        if len(parts) == 2:
            return [float(parts[0]), float(parts[1])]
    v = float(s)
    return [v, v]


def parse_vec4(val: str | None, default=(0.0, 0.0, 0.0, 0.0)):
    if not val:
        return list(default)
    s = val.strip().strip("()")
    parts = [float(p.strip()) for p in s.split(",") if p.strip()]
    while len(parts) < 4:
        parts.append(0.0)
    return parts[:4]


def parse_hsv(val: str | None):
    if not val:
        return [0.0, 0.0, 0.0]
    parts = [float(p.strip()) for p in val.split(",") if p.strip()]
    while len(parts) < 3:
        parts.append(0.0)
    return parts[:3]


def composite_image(raw_path: Path, out_rel: str) -> str:
    """Composite jpg + _alpha.png (if present) into web/assets/<out_rel>. Returns final web-relative path."""
    out_rel_norm = out_rel.replace("\\", "/").lstrip("/")
    parent = raw_path.parent
    stem = raw_path.stem
    ext = raw_path.suffix.lower()

    # Check if there is an alpha mask _<stem>.png
    alpha_candidate = None
    for child in parent.iterdir():
        if child.name.lower() == f"_{stem.lower()}.png":
            alpha_candidate = child
            break

    if ext in (".jpg", ".jpeg") and alpha_candidate is not None:
        # Composite RGB + Alpha -> .png
        out_rel_png = out_rel_norm.rsplit(".", 1)[0] + ".png"
        dest = WEB_ASSETS / out_rel_png
        if not dest.exists():
            dest.parent.mkdir(parents=True, exist_ok=True)
            rgb = Image.open(raw_path).convert("RGB")
            alpha_img = Image.open(alpha_candidate)
            if alpha_img.mode == "RGBA":
                a = alpha_img.split()[3]
            else:
                a = alpha_img.convert("L")
            if a.size != rgb.size:
                a = a.resize(rgb.size, Image.Resampling.BILINEAR)
            rgba = Image.merge("RGBA", (*rgb.split(), a))
            rgba.save(dest, "PNG", optimize=False)
        return "assets/" + out_rel_png
    else:
        dest = WEB_ASSETS / out_rel_norm
        if not dest.exists():
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(raw_path, dest)
        return "assets/" + out_rel_norm


def collect_anim_frames(anim_path_str: str):
    """Collect animation states (idle, swim, turn, eat, etc.) and composite all frames."""
    anim_dir = find_raw_ci(anim_path_str)
    if anim_dir is None or not anim_dir.is_dir():
        return {}

    states = {}
    # Find subdirectories (idle, swim, turn, eat, inflate, deflate, open, close, etc.)
    for sub in sorted(anim_dir.iterdir()):
        if not sub.is_dir():
            continue
        state_name = sub.name.lower()
        # Group frame files, ignoring _*.png alpha masks
        frames = []
        for f in sub.iterdir():
            if not f.is_file():
                continue
            if f.name.startswith("_"):
                continue
            if f.suffix.lower() not in (".jpg", ".jpeg", ".png"):
                continue
            # Extract numeric frame index if present (e.g., swim_cycle.12.jpg -> 12)
            m = re.search(r'\.(\d+)\.(?:jpg|jpeg|png)$', f.name, re.IGNORECASE)
            idx = int(m.group(1)) if m else 0
            rel = f.relative_to(RAW).as_posix()
            web_path = composite_image(f, rel)
            with Image.open(f) as im:
                w, h = im.size
            frames.append((idx, f.name, web_path, w, h))

        frames.sort(key=lambda x: (x[0], x[1]))
        if frames:
            states[state_name] = [
                {"path": wp, "w": w, "h": h} for (_, _, wp, w, h) in frames
            ]

    # Also check hudimage.png in anim_dir if present
    for f in anim_dir.iterdir():
        if f.is_file() and f.name.lower() == "hudimage.png":
            rel = f.relative_to(RAW).as_posix()
            composite_image(f, rel)

    return states


def parse_layout(layout_rel: str):
    p = find_raw_ci(layout_rel)
    if not p:
        return []
    root = parse_lenient_xml(p.read_text(encoding="latin1"))
    if not root:
        return []
    items = []
    for ch in root["children"]:
        if ch["tag"].lower() == "image":
            file_attr = ch["attrs"].get("file", "")
            raw_img = find_raw_ci(file_attr)
            if not raw_img:
                continue
            rel = raw_img.relative_to(RAW).as_posix()
            web_path = composite_image(raw_img, rel)
            with Image.open(raw_img) as im:
                w, h = im.size
            items.append({
                "file": web_path,
                "x": float(ch["attrs"].get("x", 0)),
                "y": float(ch["attrs"].get("y", 0)),
                "scaleX": float(ch["attrs"].get("scaleX", 1)),
                "scaleY": float(ch["attrs"].get("scaleY", 1)),
                "w": w,
                "h": h,
            })
    return items


def parse_fish_config(rel_path: str):
    p = find_raw_ci(rel_path)
    if not p:
        return None
    root = parse_lenient_xml(p.read_text(encoding="latin1"))
    if not root:
        return None
    a = root["attrs"]
    anim_path = a.get("animPath", "").replace("\\", "/")
    reacts = []
    for ch in root["children"]:
        if ch["tag"].lower() == "react":
            reacts.append({
                "class": ch["attrs"].get("class", ""),
                "reaction": ch["attrs"].get("reaction", "avoid"),
                "reactFreq": float(ch["attrs"].get("reactFreq", 0.5)),
            })
    return {
        "name": a.get("name", p.stem),
        "class": a.get("class", "SoldierFish"),
        "foodValue": float(a.get("foodValue", 10)),
        "maxSpeedX": parse_range(a.get("maxSpeedX"), (100.0, 150.0)),
        "maxSpeedY": parse_range(a.get("maxSpeedY"), (50.0, 80.0)),
        "minSpeedX": parse_range(a.get("minSpeedX"), (0.0, 0.0)),
        "minSpeedY": parse_range(a.get("minSpeedY"), (0.0, 0.0)),
        "sightDist": float(a.get("sightDist", 150)),
        "accelRate": float(a.get("accelRate", 400)),
        "drag": float(a.get("drag", a.get("dragRate", 40))),
        "fluidDrag": float(a.get("fluidDrag", 0.0001)),
        "scale": float(a.get("scale", 1.0)),
        "size": float(a.get("size", 1.0)),
        "hotSpot": parse_vec4(a.get("hotSpot"), (0, 0, 0, 0)),
        "collision": parse_vec4(a.get("collision"), (0, 0, 80, 80)),
        "spawnType": a.get("spawnType", "offStageLR"),
        "animPath": anim_path,
        "hsvOffset": parse_hsv(a.get("hsvOffset")),
        "reacts": reacts,
    }


def parse_time_seconds(t: str | None) -> float:
    if not t:
        return 0.0
    if ":" in t:
        parts = t.split(":")
        return float(parts[0]) * 60.0 + float(parts[1])
    return float(t)


def main():
    WEB_ASSETS.mkdir(parents=True, exist_ok=True)
    OUT_JSON.parent.mkdir(parents=True, exist_ok=True)

    manifest_path = find_raw_ci("config/stages/stageManifest.xml")
    manifest_root = parse_lenient_xml(manifest_path.read_text(encoding="latin1"))

    manifest = []
    cur_char = "Andy"
    for ch in manifest_root["children"]:
        if ch["tag"] == "fishChange":
            name = ch["attrs"].get("name", "")
            if name and name != "END":
                cur_char = name
        elif ch["tag"] == "stage":
            pos = [float(x) for x in ch["attrs"].get("mapPos", "0,0").split(",")]
            manifest.append({
                "name": ch["attrs"].get("name", ""),
                "path": ch["attrs"].get("path", ""),
                "character": cur_char,
                "mapPos": pos,
            })

    stages = {}
    fish_configs = {}
    sprites = {}

    def ensure_fish(cfg_rel: str):
        key = cfg_rel.replace("\\", "/").lower()
        if key not in fish_configs:
            cfg = parse_fish_config(cfg_rel)
            if cfg:
                fish_configs[key] = cfg
                ap = cfg["animPath"].lower()
                if ap and ap not in sprites:
                    sprites[ap] = collect_anim_frames(cfg["animPath"])
        return key

    for st in manifest:
        st_rel = f"config/stages/{st['path']}/stage.xml"
        st_p = find_raw_ci(st_rel)
        if not st_p:
            print("MISSING STAGE:", st_rel)
            continue
        root = parse_lenient_xml(st_p.read_text(encoding="latin1"))
        a = root["attrs"]

        bg_images = []
        mg_images = []
        fg_images = []
        player_cfg_key = ""
        player_sizes = []
        enemies = []
        music_file = ""

        for ch in root["children"]:
            tag = ch["tag"].lower()
            ca = ch["attrs"]
            if tag == "environment":
                if "bgLayout" in ca:
                    bg_images = parse_layout(ca["bgLayout"])
                if "mgLayout" in ca:
                    mg_images = parse_layout(ca["mgLayout"])
                if "fgLayout" in ca:
                    fg_images = parse_layout(ca["fgLayout"])
            elif tag == "playerfish":
                player_cfg_key = ensure_fish(ca.get("config", ""))
                for sz in ch["children"]:
                    if sz["tag"].lower() == "size":
                        sza = sz["attrs"]
                        player_sizes.append({
                            "size": float(sza.get("size", 1.5)),
                            "targetScore": float(sza.get("targetScore", 0)),
                            "scale": float(sza.get("scale", 1.0)),
                        })
            elif tag == "enemyfish":
                tpl_key = ensure_fish(ca.get("template", ""))
                spawns = []
                for si in ch["children"]:
                    if si["tag"].lower() == "spawninfo":
                        sia = si["attrs"]
                        spawns.append({
                            "playerSize": float(sia.get("playerSize", 0)),
                            "maxCount": int(float(sia.get("maxCount", 1))),
                            "respawnFreq": parse_range(sia.get("respawnFreq"), (1.0, 2.0)),
                        })
                enemies.append({
                    "name": ca.get("name", ""),
                    "template": tpl_key,
                    "poolSize": int(float(ca.get("poolSize", 10))),
                    "spawnInfo": spawns,
                })
            elif tag == "music":
                music_file = ca.get("gameMusic", "").replace("\\", "/")

        stages[st["path"]] = {
            "name": a.get("name", st["name"]),
            "friendlyName": a.get("friendlyName", st["name"]),
            "nextLevelAt": float(a.get("nextLevelAt", 1000)),
            "stageWidth": float(a.get("stageWidth", 800)),
            "stageHeight": float(a.get("stageHeight", 600)),
            "time": parse_time_seconds(a.get("time")),
            "stageType": a.get("type", "normal"),
            "bgImages": bg_images,
            "mgImages": mg_images,
            "fgImages": fg_images,
            "playerConfig": player_cfg_key,
            "playerSizes": player_sizes,
            "enemies": enemies,
            "music": music_file,
        }

    # Also ensure bonus bubble / special fish configs referenced at runtime are included
    for extra in [
        "config/fish/bonusbubble2x.xml",
        "config/fish/bonusbubblefeedingfury.xml",
        "config/fish/bonusbubblefreelife.xml",
        "config/fish/bonusbubbleshield.xml",
        "config/fish/bonusbubblespeed.xml",
        "config/fish/bonusbubblestar.xml",
        "config/fish/bonusbubblestun.xml",
        "config/fish/oyster.xml",
        "config/fish/mine.xml",
        "config/fish/mermaid.xml",
        "config/fish/goldenminnow.xml",
        "config/fish/brineshrimp.xml",
    ]:
        ensure_fish(extra)

    # Copy audio files (sounds & music)
    for folder in ["sounds", "music"]:
        src_dir = GAME_RES / folder
        if src_dir.is_dir():
            dst_dir = WEB_ASSETS / "resources" / folder
            dst_dir.mkdir(parents=True, exist_ok=True)
            for wav in src_dir.glob("*.wav"):
                dst = dst_dir / wav.name
                if not dst.exists():
                    shutil.copy2(wav, dst)

    # Copy splash image if present
    splash = GAME_RES / "splash" / "gameSplash.jpg"
    if splash.exists():
        dst = WEB_ASSETS / "resources" / "splash" / "gameSplash.jpg"
        dst.parent.mkdir(parents=True, exist_ok=True)
        if not dst.exists():
            shutil.copy2(splash, dst)

    # Copy map, menu, and HUD images
    for ui_img in ["resources/map/mapBG.jpg", "resources/menus/mainMenuBG.jpg"]:
        raw_ui = find_raw_ci(ui_img)
        if raw_ui:
            composite_image(raw_ui, raw_ui.relative_to(RAW).as_posix())

    hud_dir = find_raw_ci("resources/hud")
    if hud_dir and hud_dir.is_dir():
        for f in hud_dir.iterdir():
            if f.is_file() and not f.name.startswith("_") and f.suffix.lower() in (".jpg", ".jpeg", ".png"):
                composite_image(f, f.relative_to(RAW).as_posix())

    data = {
        "manifest": manifest,
        "stages": stages,
        "fishConfigs": fish_configs,
        "sprites": sprites,
    }
    OUT_JSON.write_text(json.dumps(data, separators=(",", ":")), encoding="utf-8")
    print(
        f"Done! {len(manifest)} stages, {len(fish_configs)} fish configs, "
        f"{len(sprites)} sprite sets -> {OUT_JSON} ({OUT_JSON.stat().st_size // 1024} KB)"
    )


if __name__ == "__main__":
    main()
