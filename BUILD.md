# Build

Rust is stable from rustup. The browser build also needs the `wasm32-unknown-unknown` target and wasm-bindgen `0.2.130` (the same version as `crates/engine`).

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.130 --locked
```

A prebuilt `wasm-bindgen` 0.2.130 binary is fine. The CLI version has to match the crate.

## Browser

```sh
cargo build -p section-engine --release --target wasm32-unknown-unknown --features wasm
wasm-bindgen --target web --out-dir web/public/engine \
  target/wasm32-unknown-unknown/release/section_engine.wasm

cd web
npm install
npm run dev
```

`npm run build` writes `web/dist` for https://section-cyan.vercel.app/. The Vite base is `/`. `prebuild` copies `MANUAL.md` into `web/public/` so the Manual link works.

Vercel builds the `web/` directory. It does not compile Rust, so `web/public/engine` (the wasm-bindgen output) is committed. After an engine change, rerun the two commands above and commit that folder.

`target/`, `web/node_modules`, `web/dist`, and the copied `web/public/MANUAL.md` are generated. Do not commit them.

## Plugin

```sh
cargo xtask bundle section --release
```

That writes `target/bundled/section.vst3` and ad-hoc signs it. Install:

```sh
rm -rf ~/Library/Audio/Plug-Ins/VST3/section.vst3
cp -R target/bundled/section.vst3 ~/Library/Audio/Plug-Ins/VST3/
```

In Ableton Live, rescan plug-ins, then load Section on a MIDI track. Channel 10 plays drums. Any other channel plays bass. Live’s transport sets the tempo and starts the band.

## Tests

```sh
cargo test -p section-engine --lib
```
