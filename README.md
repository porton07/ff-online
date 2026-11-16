# Feeding Frenzy — Rust + WebAssembly (WASM)

A Rust + WebAssembly web port of **Feeding Frenzy** with all 40 original stages, 5 playable fish (Andy, Leon, Eddie, JD, Orville), original sprites/audio, and **Online Co-Op (2–4 players)** with player-vs-player eating.

## Run Locally

```bash
node server.js
```

Open `http://localhost:8080/` in your browser.

## Build WASM from Source

```bash
rustup target add wasm32-unknown-unknown
cargo build --manifest-path crates/game/Cargo.toml --target wasm32-unknown-unknown --release
cp crates/game/target/wasm32-unknown-unknown/release/feeding_frenzy_wasm.wasm web/feeding_frenzy_wasm.wasm
```

## Controls

- **Mouse** or **WASD / Arrow Keys**: Swim
- **Left Click / Space / Shift**: Dash
- **Right Click / E**: John Dory Suction (Stages 25–32)
- **Esc / P**: Pause
- **Online Co-Op**: Click **Create Online Co-Op** to get a 4-letter room code, or enter a code and click **Join**.
