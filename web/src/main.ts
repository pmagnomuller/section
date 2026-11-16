const PAD_KEYS = ["a", "s", "d", "f", "g", "h", "j", "z", "x", "c", "v", "b", "n"];
const PAD_LABELS = ["KICK", "SNARE", "HAT", "OPEN", "RIM", "BELL", "TAMB", "HI TOM", "MID", "LO TOM", "RIDE", "CRASH", "CLAP"];
const NOTE_NAMES = ["C", "C#", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B"];
const SPACES = ["Room", "Hall", "Plate", "Spring", "Delay", "Chorus", "Flanger"];
const STORE = "section-doc-v1";

type Hud = {
  mode: number;
  playing: boolean;
  recording: boolean;
  tempo: number;
  key: number;
  keyName: string;
  swing: number;
  bassMute: boolean;
  metronome: boolean;
  drumPart: boolean;
  octave: number;
  quantize: number;
  pattern: number;
  patternName: string;
  song: number;
  songName: string;
  kit: number;
  kitName: string;
  bass: number;
  bassName: string;
  bar: number;
  step: number;
  steps: number;
  peak: number;
  grid: number[][];
  bassGrid: number[];
  songPads: string[];
  fx: Fx;
};

type Fx = {
  comp_on: boolean;
  comp: number;
  tone: number;
  resonance: number;
  res_freq: number;
  lofi: number;
  space_on: boolean;
  space: number;
  space_mix: number;
  space_time: number;
};

type Catalog = { songs: string[]; patterns: string[]; kits: string[]; basses: string[] };

const app = document.querySelector("#app")!;
app.innerHTML = `
  <main class="machine">
    <header>
      <div>
        <h1 class="brand">Section</h1>
        <p class="tag">RT-223-style rhythm section for jamming. Mute the synth bass and use it as a drum machine with guitar or bass.</p>
      </div>
      <div class="header-links">
        <button type="button" class="manual-toggle" id="manual-toggle" aria-expanded="false">Manual</button>
        <a class="home" href="https://pedro-muller.com/">Pedro Müller</a>
      </div>
    </header>
    <section class="manual" id="manual" hidden>
      <h2>Like a Zoom RhythmTrak RT-223</h2>
      <p>
        Song mode with Intro / Verse / Chorus / Fill / Break / End, Pattern for the grid,
        Groove for stacking loops. Sounds and patterns are original — not a Zoom product.
      </p>
      <h2>Jam with guitar or bass</h2>
      <ol>
        <li>Click <b>Play</b> once so the browser can start audio.</li>
        <li>Click <b>Bass on</b> so it becomes <b>Bass off</b>, or press <kbd>\\</kbd>. Drums keep going; synth bass stops.</li>
        <li>Pick a style. For guitar jams: Rock, Indie Rock, Alt Rock, Blues, Blues Shuffle.</li>
        <li>Set tempo with <b>Tempo − / +</b> or <kbd>[</kbd> <kbd>]</kbd>.</li>
        <li>Stay in <b>Song</b>. Press <kbd>Space</kbd> or hit sections <b>1–6</b> like the RT-223 song pads: Intro, Verse, Chorus, Fill, Break, End.</li>
        <li>Hit pads for live accents. Keys: <kbd>A S D F G H J</kbd> and <kbd>Z X C V B N</kbd>.</li>
      </ol>
      <p>
        Song is for playing along. Pattern is for rewriting the step grid.
        Record writes into the pattern you were hearing. Save keeps edits in this browser; Factory restores the stock songs.
      </p>
      <p class="manual-more">
        Full walkthrough and RT-223 map:
        <a href="/drum-machine/MANUAL.md">MANUAL.md</a>
        ·
        <a href="https://github.com/pmagnomuller/section/blob/master/MANUAL.md">on GitHub</a>
      </p>
    </section>
    <section class="lcd" id="lcd"></section>
    <p class="status" id="status">Mute bass, pick a style, then play. Click play or a pad to start audio.</p>
    <div class="transport" id="transport"></div>
    <div class="songs" id="songs"></div>
    <div class="sections" id="sections"></div>
    <div class="pads" id="pads"></div>
    <div class="tune" id="tune"></div>
    <div class="grid" id="grid"></div>
    <div class="fx" id="fx"></div>
    <p class="help">
      For drums-only accompaniment: mute the synth bass with <b>Bass on</b> or <kbd>\\</kbd>.
      Sections 1–6 change the form. Pads: A S D F G H J and Z X C V B N.
      Space plays and stops. [ ] tempo. Tab switches drums and bass pads. R records.
    </p>
  </main>
`;

document.querySelector("#manual-toggle")!.addEventListener("click", () => {
  const panel = document.querySelector("#manual") as HTMLElement;
  const toggle = document.querySelector("#manual-toggle") as HTMLButtonElement;
  const open = panel.hasAttribute("hidden");
  if (open) panel.removeAttribute("hidden");
  else panel.setAttribute("hidden", "");
  toggle.setAttribute("aria-expanded", open ? "true" : "false");
  toggle.classList.toggle("on", open);
});

let ctx: AudioContext | null = null;
let handle: {
  cmd(json: string): void;
  hud(): string;
  catalog(): string;
  export_doc(): string;
  render(frames: number): number;
} | null = null;
let hud: Hud | null = null;
let catalog: Catalog | null = null;
let ready: Promise<void> | null = null;
let lastSong = -1;
let lastPattern = -1;
let lastSteps = -1;
const held = new Set<string>();

function send(command: object) {
  handle?.cmd(JSON.stringify(command));
}

async function ensureAudio() {
  if (handle && ctx) {
    if (ctx.state !== "running") await ctx.resume();
    return;
  }
  setStatus("Starting audio…");
  if (!ready) ready = boot();
  await ready;
}

async function boot() {
  try {
    await bootAudio();
  } catch (error) {
    setStatus(error instanceof Error ? error.message : String(error));
    throw error;
  }
}

async function bootAudio() {
  const context = new AudioContext();
  ctx = context;
  const base = import.meta.env.BASE_URL;
  await context.audioWorklet.addModule(`${base}player-worklet.js`);
  const engineUrl = new URL(`${base}engine/section_engine.js`, window.location.href);
  const engine = await import(/* @vite-ignore */ engineUrl.href);
  await engine.default();
  const memory = engine.take_memory();
  handle = new engine.Handle(context.sampleRate);
  const worklet = new AudioWorkletNode(context, "player", { outputChannelCount: [2] });
  worklet.connect(context.destination);
  let queued = 0;
  worklet.port.onmessage = (event) => {
    if (event.data?.type === "queued") queued = event.data.frames;
  };
  window.setInterval(() => {
    if (!handle) return;
    const target = context.sampleRate * 0.18;
    while (queued < target) {
      const frames = 256;
      const ptr = handle.render(frames);
      const view = new Float32Array(memory.buffer, ptr, frames * 2);
      const copy = new Float32Array(view);
      queued += frames;
      worklet.port.postMessage({ type: "pcm", buffer: copy }, [copy.buffer]);
    }
  }, 20);
  const saved = localStorage.getItem(STORE);
  if (saved) send({ t: "load_document", json: saved });
  catalog = JSON.parse(handle.catalog());
  window.setInterval(() => {
    if (!handle) return;
    hud = JSON.parse(handle.hud());
    paint();
  }, 80);
  if (context.state !== "running") await context.resume();
}

function setStatus(text: string) {
  const el = document.querySelector("#status");
  if (el) el.textContent = text;
}

function paint() {
  if (!hud) return;
  paintLcd();
  paintTransport();
  paintSongs();
  paintSections();
  paintPads();
  paintTune();
  if (hud.song !== lastSong || hud.pattern !== lastPattern || hud.steps !== lastSteps) {
    lastSong = hud.song;
    lastPattern = hud.pattern;
    lastSteps = hud.steps;
  }
  paintGrid();
  paintFx();
  const mode = ["Pattern", "Song", "Groove"][hud.mode] ?? "Song";
  setStatus(hud.playing ? `${mode} · ${hud.songName} is playing` : `${mode} · stopped`);
}

function paintLcd() {
  const h = hud!;
  const lcd = document.querySelector("#lcd")!;
  const peak = Math.min(100, Math.round(h.peak * 140));
  lcd.innerHTML = `
    <div><div class="big"><b>${escape(h.songName)}</b></div><div>${escape(h.patternName)} · bar ${h.bar}</div></div>
    <div><div class="big">${h.tempo.toFixed(1)}</div><div>${h.keyName} · ${escape(h.kitName)}</div></div>
    <div><div>${escape(h.bassName)}${h.bassMute ? " muted" : ""}</div><div class="meter"><span style="width:${peak}%"></span></div></div>
  `;
}

function paintTransport() {
  const h = hud ?? {
    playing: false,
    recording: false,
    mode: 1,
    drumPart: true,
    bassMute: false,
    metronome: false,
  };
  const root = document.querySelector("#transport")!;
  if (!root.childElementCount) {
    button(root, "Play", false, () => send(hud?.playing ? { t: "stop" } : { t: "play" }));
    button(root, "Record", false, () => send({ t: "toggle_rec" }));
    button(root, "Pattern", false, () => send({ t: "set_mode", mode: 0 }));
    button(root, "Song", false, () => send({ t: "set_mode", mode: 1 }));
    button(root, "Groove", false, () => send({ t: "set_mode", mode: 2 }));
    button(root, "Drums", false, () => send({ t: "set_part", drum: !hud?.drumPart }));
    button(root, "Bass on", false, () => send({ t: "set_bass_mute", mute: !hud?.bassMute }));
    button(root, "Click", false, () => send({ t: "set_metronome", on: !hud?.metronome }));
  }
  const labels = [
    h.playing ? "Stop" : "Play",
    h.recording ? "Recording" : "Record",
    "Pattern",
    "Song",
    "Groove",
    h.drumPart ? "Drums" : "Bass",
    h.bassMute ? "Bass off" : "Bass on",
    h.metronome ? "Click on" : "Click",
  ];
  const on = [h.playing, h.recording, h.mode === 0, h.mode === 1, h.mode === 2, !h.drumPart, h.bassMute, h.metronome];
  [...root.children].forEach((child, index) => {
    child.textContent = labels[index];
    child.classList.toggle("on", on[index]);
  });
}

function paintSongs() {
  const root = document.querySelector("#songs")!;
  if (!catalog || (root.childElementCount === catalog.songs.length && root.dataset.song === String(hud?.song))) {
    markCurrent(root, hud?.song ?? 0);
    return;
  }
  root.innerHTML = "";
  root.dataset.song = String(hud?.song ?? 0);
  catalog.songs.forEach((name, index) => {
    button(root, name, index === hud?.song, () => send({ t: "set_song", index }));
  });
}

function paintSections() {
  const root = document.querySelector("#sections")!;
  const pads = hud?.songPads?.length
    ? hud.songPads
    : ["INTRO", "VERSE", "CHORUS", "FILL", "BREAK", "END"];
  if (root.childElementCount !== pads.length) {
    root.innerHTML = "";
    pads.forEach((label, index) => {
      const el = button(root, `${index + 1}  ${label}`, false, () => send({ t: "section", pad: index }));
      el.dataset.section = String(index);
    });
  } else {
    pads.forEach((label, index) => {
      const el = root.children[index] as HTMLButtonElement;
      el.textContent = `${index + 1}  ${label}`;
    });
  }
}

function paintPads() {
  const root = document.querySelector("#pads")!;
  if (root.childElementCount === 13) {
    [...root.children].forEach((child, index) => {
      const buttonEl = child as HTMLButtonElement;
      const key = PAD_KEYS[index].toUpperCase();
      buttonEl.innerHTML = `${padTitle(index)}<small>${key}</small>`;
    });
    return;
  }
  root.innerHTML = "";
  PAD_LABELS.forEach((_, index) => {
    const el = document.createElement("button");
    el.innerHTML = `${padTitle(index)}<small>${PAD_KEYS[index].toUpperCase()}</small>`;
    el.addEventListener("pointerdown", (event) => {
      event.preventDefault();
      const rect = el.getBoundingClientRect();
      const vel = Math.min(1, Math.max(0.35, 1 - (event.clientY - rect.top) / rect.height));
      el.classList.add("hit");
      void ensureAudio().then(() => {
        if (hud?.mode === 2) send({ t: "groove", pad: index });
        else send({ t: "pad_down", pad: index, vel });
      });
    });
    const up = () => {
      el.classList.remove("hit");
      send({ t: "pad_up", pad: index });
    };
    el.addEventListener("pointerup", up);
    el.addEventListener("pointerleave", up);
    root.append(el);
  });
}

function paintTune() {
  const root = document.querySelector("#tune")!;
  const h = hud!;
  const signature = `${h.kit}:${h.bass}:${h.pattern}:${h.octave}:${catalog?.patterns.length ?? 0}`;
  if (root.dataset.signature === signature) return;
  root.dataset.signature = signature;
  root.innerHTML = "";
  button(root, "Tempo −", false, () => send({ t: "set_tempo", bpm: Math.max(40, (hud?.tempo ?? 120) - 1) }));
  button(root, "Tempo +", false, () => send({ t: "set_tempo", bpm: Math.min(280, (hud?.tempo ?? 120) + 1) }));
  button(root, "Key −", false, () => send({ t: "set_key", key: ((hud?.key ?? 0) + 11) % 12 }));
  button(root, "Key +", false, () => send({ t: "set_key", key: ((hud?.key ?? 0) + 1) % 12 }));
  button(root, "Oct −", false, () => send({ t: "set_octave", octave: Math.max(-2, h.octave - 1) }));
  button(root, "Oct +", false, () => send({ t: "set_octave", octave: Math.min(2, h.octave + 1) }));
  if (catalog) {
    root.append(selectOf("Kit", catalog.kits, h.kit, (index) => send({ t: "set_kit", index })));
    root.append(selectOf("Bass", catalog.basses, h.bass, (index) => send({ t: "set_bass", index })));
    root.append(selectOf("Pattern", catalog.patterns, h.pattern, (index) => send({ t: "set_pattern", index })));
  }
  button(root, "Clear drums", false, () => send({ t: "clear_drums" }));
  button(root, "Clear bass", false, () => send({ t: "clear_bass" }));
  button(root, "Duplicate", false, () => {
    send({ t: "duplicate_pattern" });
    if (handle) catalog = JSON.parse(handle.catalog());
    paint();
  });
  button(root, "Save", false, () => {
    if (!handle) return;
    localStorage.setItem(STORE, handle.export_doc());
    setStatus("Saved in this browser.");
  });
  button(root, "Factory", false, () => {
    localStorage.removeItem(STORE);
    send({ t: "reset_factory" });
    if (handle) catalog = JSON.parse(handle.catalog());
    paint();
  });
}

function paintGrid() {
  const root = document.querySelector("#grid") as HTMLElement;
  const h = hud!;
  const steps = Math.min(32, h.steps || 16);
  root.style.setProperty("--steps", String(steps));
  const signature = `${h.pattern}:${steps}:${h.drumPart}`;
  if (root.dataset.signature !== signature) {
    root.dataset.signature = signature;
    root.innerHTML = "";
    for (let pad = 0; pad < 13; pad += 1) {
      const row = document.createElement("div");
      row.className = "grid-row";
      const label = document.createElement("div");
      label.className = "cell-label";
      label.textContent = PAD_LABELS[pad];
      row.append(label);
      for (let step = 0; step < steps; step += 1) {
        const cell = document.createElement("button");
        cell.dataset.pad = String(pad);
        cell.dataset.step = String(step);
        cell.addEventListener("click", () => send({ t: "toggle_step", pad, step, vel: 100 }));
        row.append(cell);
      }
      root.append(row);
    }
    const bass = document.createElement("div");
    bass.className = "grid-row";
    const label = document.createElement("div");
    label.className = "cell-label";
    label.textContent = "BASS";
    bass.append(label);
    for (let step = 0; step < steps; step += 1) {
      const cell = document.createElement("button");
      cell.className = "bass-cell";
      cell.dataset.bass = String(step);
      cell.addEventListener("click", () => {
        const current = hud?.bassGrid[step] ?? -1;
        if (current >= 0) send({ t: "set_bass_step", step, note: -1, dur: 1 });
        else {
          const rootNote = 36 + (hud?.key ?? 0) + (hud?.octave ?? 0) * 12;
          send({ t: "set_bass_step", step, note: rootNote, dur: 2 });
        }
      });
      cell.addEventListener("contextmenu", (event) => {
        event.preventDefault();
        const current = hud?.bassGrid[step] ?? -1;
        if (current >= 0) send({ t: "set_bass_step", step, note: current + 1, dur: 2 });
      });
      bass.append(cell);
    }
    root.append(bass);
  }
  const rows = [...root.querySelectorAll(".grid-row")];
  rows.slice(0, 13).forEach((row, pad) => {
    [...row.querySelectorAll("button")].forEach((cell, step) => {
      const on = (h.grid[pad]?.[step] ?? 0) > 0;
      cell.classList.toggle("on", on);
      cell.classList.toggle("playhead", step === h.step);
    });
  });
  const bassRow = rows[13];
  if (bassRow) {
    [...bassRow.querySelectorAll("button")].forEach((cell, step) => {
      const note = h.bassGrid[step];
      cell.textContent = note >= 0 ? midiName(note) : "";
      cell.classList.toggle("on", note >= 0);
      cell.classList.toggle("playhead", step === h.step);
    });
  }
}

function paintFx() {
  const root = document.querySelector("#fx")!;
  if (!hud) return;
  if (root.childElementCount > 0) {
    const fx = hud.fx;
    (root.querySelector("[data-k='comp']") as HTMLInputElement).value = String(fx.comp);
    (root.querySelector("[data-k='tone']") as HTMLInputElement).value = String(fx.tone);
    (root.querySelector("[data-k='space_mix']") as HTMLInputElement).value = String(fx.space_mix);
    (root.querySelector("[data-k='swing']") as HTMLInputElement).value = String(hud.swing);
    return;
  }
  const fx = hud.fx;
  root.append(slider("Comp", fx.comp, 0, 1, 0.01, "comp", (value) => updateFx({ comp: value, comp_on: value > 0.01 })));
  root.append(slider("Tone", fx.tone, -1, 1, 0.01, "tone", (value) => updateFx({ tone: value })));
  root.append(slider("Space", fx.space_mix, 0, 1, 0.01, "space_mix", (value) => updateFx({ space_mix: value, space_on: value > 0.01 })));
  root.append(slider("Swing", hud.swing, 0, 0.7, 0.01, "swing", (value) => send({ t: "set_swing", amount: value })));
  const space = document.createElement("select");
  SPACES.forEach((name, index) => {
    const option = document.createElement("option");
    option.value = String(index);
    option.textContent = name;
    option.selected = index === fx.space;
    space.append(option);
  });
  space.addEventListener("change", () => updateFx({ space: Number(space.value), space_on: true }));
  root.append(space);
}

function updateFx(patch: Partial<Fx>) {
  if (!hud) return;
  const fx = { ...hud.fx, ...patch };
  hud.fx = fx;
  send({ t: "set_fx", fx });
}

function button(parent: Element, label: string, on: boolean, action: () => void) {
  const el = document.createElement("button");
  el.textContent = label;
  el.classList.toggle("on", on);
  el.addEventListener("click", () => {
    void ensureAudio().then(action);
  });
  parent.append(el);
  return el;
}

function markCurrent(root: Element, index: number) {
  [...root.children].forEach((child, i) => child.classList.toggle("on", i === index));
}

function selectOf(label: string, options: string[], current: number, onChange: (index: number) => void) {
  const wrap = document.createElement("label");
  wrap.textContent = label;
  const select = document.createElement("select");
  options.forEach((name, index) => {
    const option = document.createElement("option");
    option.value = String(index);
    option.textContent = name;
    option.selected = index === current;
    select.append(option);
  });
  select.addEventListener("change", () => onChange(Number(select.value)));
  wrap.append(select);
  return wrap;
}

function slider(label: string, value: number, min: number, max: number, step: number, key: string, onInput: (value: number) => void) {
  const wrap = document.createElement("label");
  wrap.textContent = label;
  const input = document.createElement("input");
  input.type = "range";
  input.min = String(min);
  input.max = String(max);
  input.step = String(step);
  input.value = String(value);
  input.dataset.k = key;
  input.addEventListener("input", () => onInput(Number(input.value)));
  wrap.append(input);
  return wrap;
}

function padTitle(index: number) {
  if (hud?.mode === 2) return hud.songPads[index] ?? "—";
  if (hud?.drumPart === false) return bassName(index);
  return PAD_LABELS[index];
}

function bassName(pad: number) {
  if (!hud) return PAD_LABELS[pad];
  return midiName(36 + hud.key + hud.octave * 12 + pad);
}

function midiName(note: number) {
  const name = NOTE_NAMES[((note % 12) + 12) % 12];
  const octave = Math.floor(note / 12) - 1;
  return `${name}${octave}`;
}

function escape(value: string) {
  return value.replace(/[&<>]/g, (char) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;" })[char]!);
}

window.addEventListener("keydown", (event) => {
  if (event.repeat || event.metaKey || event.ctrlKey || event.altKey) return;
  const key = event.key.toLowerCase();
  if (key === " ") {
    event.preventDefault();
    void ensureAudio().then(() => send(hud?.playing ? { t: "stop" } : { t: "play" }));
    return;
  }
  if (key >= "1" && key <= "6") {
    event.preventDefault();
    void ensureAudio().then(() => send({ t: "section", pad: Number(key) - 1 }));
    return;
  }
  const pad = PAD_KEYS.indexOf(key);
  if (pad >= 0 && !held.has(key)) {
    event.preventDefault();
    held.add(key);
    void ensureAudio().then(() => {
      const vel = event.shiftKey ? 0.45 : 0.9;
      if (hud?.mode === 2) send({ t: "groove", pad });
      else send({ t: "pad_down", pad, vel });
      document.querySelectorAll("#pads button")[pad]?.classList.add("hit");
    });
    return;
  }
  if (key === "[") send({ t: "set_tempo", bpm: Math.max(40, (hud?.tempo ?? 120) - 1) });
  if (key === "]") send({ t: "set_tempo", bpm: Math.min(280, (hud?.tempo ?? 120) + 1) });
  if (key === "-" || key === "_") send({ t: "set_key", key: ((hud?.key ?? 0) + 11) % 12 });
  if (key === "=" || key === "+") send({ t: "set_key", key: ((hud?.key ?? 0) + 1) % 12 });
  if (key === "tab") {
    event.preventDefault();
    send({ t: "set_part", drum: !hud?.drumPart });
  }
  if (key === "\\") send({ t: "set_bass_mute", mute: !hud?.bassMute });
  if (key === "r") send({ t: "toggle_rec" });
  if (key === "m") send({ t: "set_metronome", on: !hud?.metronome });
});

window.addEventListener("keyup", (event) => {
  const key = event.key.toLowerCase();
  if (!held.has(key)) return;
  held.delete(key);
  const pad = PAD_KEYS.indexOf(key);
  if (pad >= 0) {
    send({ t: "pad_up", pad });
    document.querySelectorAll("#pads button")[pad]?.classList.remove("hit");
  }
});

document.body.addEventListener("pointerdown", () => {
  void ensureAudio();
}, { once: true });

paintTransport();
paintSections();
paintPads();
