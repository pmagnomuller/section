# Section

A drum and bass rhythm section for the MacBook: play it in the browser, and load the same instrument in Ableton Live. Sounds, patterns, and names are original. This is the workflow of a hardware rhythm machine (pads, kits, bass, pattern / song / groove), not a copy of any product.

## State

Working on this machine:

- One Rust engine (`crates/engine`) shared by the VST and the browser. 16 kits, 12 bass programs, 14 styles with intro / verse / chorus / fill / break / end, song mode, swing, key, and effects.
- Engine tests pass (`cargo test -p section-engine --lib`).
- VST3 **Section** is built and installed at `~/Library/Audio/Plug-Ins/VST3/Section.vst3`, ad-hoc signed. MIDI channel 10 plays drums. Any other channel plays bass. While Live’s transport is running, the plugin follows the host tempo and position. Vendor Pedro Muller.
- The browser app (`web/`) plays locally. Play starts the Funk song, Stop stops it, and the page has sections, pads, kits, bass, the step grid, and effects.
- Browser audio renders the wasm engine on the main thread and sends buffers to `web/public/player-worklet.js`. The wasm-bindgen glue does not load inside an AudioWorklet, so the engine stays off the audio thread.
- Generated wasm (`web/public/engine`), `target/`, `web/node_modules`, and `web/dist` are gitignored. Rebuild them; do not commit them.

Not done yet:

- Nothing is on pedro-muller.com. The Astro site still has no `/drum-machine` page and no nav link.
- Ableton has not been rescanned in this session, so Live may not list Section until Plug-Ins is rescanned.
- Recording, groove mode, and Save / Factory exist in the UI and have not been walked through end to end.
- There is no build or install note beyond this file.

## Plan

1. Rescan plug-ins in Ableton Live 11 and load Section on a MIDI track. Press Play in Live and confirm the band follows the transport. Stop Live and confirm the plugin stops. Try a section button and a pad.
2. Publish the browser build at https://pedro-muller.com/drum-machine.
   - Rebuild wasm into `web/public/engine` (`wasm-bindgen` 0.2.130, matching the crate).
   - Production Vite base is `/drum-machine/`.
   - Copy the build into the Astro site at `public/drum-machine/` so it is served as static files.
   - Add a nav item. The site uses trailing slashes.
3. On the live page, play a song, change section, hit a pad, and stop.
4. Walk recording, groove mode, and Save / Factory once in the browser.
5. Add a short build note: rustup stable, `wasm32-unknown-unknown`, wasm-bindgen 0.2.130, `cargo build -p section --release`, codesign the bundle, `npm run dev` and `npm run build`.

## Local browser

```sh
cd web
npm install
npm run dev
```

Open the URL Vite prints. The first Play or pad click starts audio.
