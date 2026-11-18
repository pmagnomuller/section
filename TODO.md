# Section

A drum and bass rhythm section for the MacBook: play it in the browser, and load the same instrument in Ableton Live. Sounds, patterns, and names are original. The workflow follows a Zoom RhythmTrak RT-223 (pads, kits, bass, pattern / song / groove), not a copy of that product.

## State

Working on this machine:

- One Rust engine (`crates/engine`) shared by the VST and the browser. 19 kits (original 16 plus Indie Room, Alt Dry, Blues Shuffle), 12 bass programs, 17 styles including Indie Rock, Alt Rock, and Blues Shuffle, with intro / verse / chorus / fill / break / end, song mode, swing, key, and effects.
- Engine tests pass (`cargo test -p section-engine --lib`), including section changes, recording into the heard pattern, groove, and factory reset.
- Record switches to pattern mode on the pattern you were hearing, then writes drums and bass into it.
- VST3 is built with `cargo xtask bundle section --release` and installed at `~/Library/Audio/Plug-Ins/VST3/section.vst3`, ad-hoc signed. MIDI channel 10 plays drums. Any other channel plays bass. While Live’s transport is running, the plugin follows the host tempo and position. Vendor Pedro Muller. This Mac has no Ableton, so Live has not loaded it.
- The browser app plays locally. Play starts Funk. Chorus, Stop, a pad, Record, Groove, Save, and Factory were walked in the browser.
- Browser audio renders the wasm engine on the main thread and sends buffers to `web/public/player-worklet.js`. The wasm-bindgen glue does not load inside an AudioWorklet, so the engine stays off the audio thread.
- The browser app is its own site at https://section.pedro-muller.com/. The personal site lists it under Projects and redirects `/drum-machine/` there.
- `README.md` covers how it works, the short user guide, and Ableton setup. `MANUAL.md` is the full RT-223-style walkthrough. The page has a Manual panel with the short path.
- `web/public/engine` is committed so Vercel can build the browser app without Rust. `target/`, `web/node_modules`, and `web/dist` stay gitignored. Commands are in `BUILD.md`.

Not done yet:

- Rescan plug-ins in Ableton on a machine that has Live, and confirm the band follows the transport.
- After the site deploy, play a song on https://section.pedro-muller.com/, change section, hit a pad, and stop.

## Plan

1. On a Mac with Ableton Live, rescan plug-ins and load Section on a MIDI track. Press Play in Live and confirm the band follows the transport. Stop Live and confirm the plugin stops. Try a section button and a pad.
2. On https://section.pedro-muller.com/, play a song, change section, hit a pad, and stop.

## Local browser

```sh
cd web
npm install
npm run dev
```

Open the URL Vite prints. The first Play or pad click starts audio.
