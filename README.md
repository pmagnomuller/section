# Section

A drum and bass rhythm machine for the browser and Ableton Live. Built for personal practice on guitar and bass, shared so other people can try the same instrument.

Live demo: https://section-cyan.vercel.app/

The workflow follows a [Zoom RhythmTrak RT-223](https://www.zoom.co.jp/sites/default/files/products/downloads/pdfs/E_RT-223.pdf): Song mode with intro / verse / chorus / fill / break / end, Pattern mode for editing, Groove for stacking loops, kits, and bass programs. Sounds and patterns are original. This is not a Zoom product and not a dump of RT-223 samples.

## How it works

One Rust audio engine (`crates/engine`) drives both front ends:

- **Browser** (`web/`): wasm build of the engine, played through an AudioWorklet buffer player
- **Ableton** (`crates/plugin/`): VST3 instrument named `section`, same engine, egui editor

You pick a **style** (Rock, Indie Rock, Alt Rock, Blues Shuffle, Funk, …). Each style has section pads and patterns. In **Song** mode you assemble a form live, the same way you would on an RT-223. Mute the synth bass when you want drums only behind your own bass and guitar.

Full walkthrough: [MANUAL.md](MANUAL.md). Build and install notes: [BUILD.md](BUILD.md).

## Quick start in the browser

1. Open https://section-cyan.vercel.app/ (or run locally; see below).
2. Click **Play** once so the browser can start audio.
3. Click **Bass on** until it reads **Bass off** (or press `\`).
4. Pick a style for guitar jams: Rock, Hard Rock, Indie Rock, Alt Rock, Blues, Blues Shuffle.
5. Set tempo with **Tempo − / +** or `[` / `]`.
6. Stay in **Song**. Use sections **1–6** as the form:
   - **1 Intro**: once, then verse
   - **2 Verse**: loops
   - **3 Chorus**: loops
   - **4 Fill**: fill, then back
   - **5 Break**: break, then back
   - **6 End**: ending, then stop
7. Hit pads for accents (`A S D F G H J` and `Z X C V B N`). **Space** stops.

## User manual (short)

| Mode | Use it for |
| --- | --- |
| **Song** | Jamming. Section pads change intro / verse / chorus / fill / break / end. |
| **Pattern** | Editing one pattern on the step grid, or recording pads into it. |
| **Groove** | Stacking section patterns from the pads. |

| Control | Action |
| --- | --- |
| Play / Stop, Space | Start or stop |
| Bass on / Bass off, `\` | Mute synth bass (use this for guitar / bass practice) |
| Record, R | Record into the pattern you were hearing |
| Kit / Bass menus | Change the drum kit or synth bass program |
| Swing | Loosen the feel |
| Save | Keep edits in this browser |
| Factory | Restore the stock songs |

Drum pads: Kick, Snare, Hat, Open, Rim, Bell, Tamb, Hi Tom, Mid Tom, Lo Tom, Ride, Crash, Clap.

The step grid under the pads edits the current pattern. The bottom **BASS** row is the synth bassline. Mute that track when you play your own bass.

More detail, keyboard map, and jam tips: [MANUAL.md](MANUAL.md). On the live page, open the **Manual** button.

## Connect it to Ableton Live

### 1. Build and install the VST3 (macOS)

You need Rust (rustup stable).

```sh
git clone https://github.com/pmagnomuller/section.git
cd section
cargo xtask bundle section --release
rm -rf ~/Library/Audio/Plug-Ins/VST3/section.vst3
cp -R target/bundled/section.vst3 ~/Library/Audio/Plug-Ins/VST3/
```

The bundle is ad-hoc signed. Details: [BUILD.md](BUILD.md).

### 2. Rescan and load

1. Open Ableton Live.
2. Preferences → Plug-Ins → Rescan (or enable VST3 folder if it was off).
3. Create a **MIDI track**.
4. Drop **section** on that track (Instruments / VST3 / Pedro Muller, or search “section”).
5. Arm the track or set Monitor to In if you want MIDI pads / a keyboard.

### 3. Play with Live’s transport

- Press **Play** in Live. Section follows host tempo and song position.
- Press **Stop** in Live. The band stops with the host.
- Use the plugin’s section pads (Intro / Verse / Chorus / Fill / Break / End) while Live is running.
- When Live is stopped, use **Play** in the plugin UI to preview.

### 4. MIDI routing

| MIDI | What it plays |
| --- | --- |
| Channel **10** | Drum kit |
| Any other channel | Synth bass |

For drums-only practice behind guitar and bass, mute bass in the plugin UI (same idea as **Bass off** in the browser).

## Local browser development

```sh
# once: wasm target + matching wasm-bindgen 0.2.130
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.130 --locked

cargo build -p section-engine --release --target wasm32-unknown-unknown --features wasm
wasm-bindgen --target web --out-dir web/public/engine \
  target/wasm32-unknown-unknown/release/section_engine.wasm

cd web
npm install
npm run dev
```

Open the URL Vite prints. The first Play or pad click starts audio.

## Repo layout

| Path | Role |
| --- | --- |
| `crates/engine` | Shared synth, sequencer, presets |
| `crates/plugin` | VST3 host |
| `web/` | Browser UI |
| `MANUAL.md` | Full user manual |
| `BUILD.md` | Toolchain and install commands |

## License

MIT. See the crate manifests.
