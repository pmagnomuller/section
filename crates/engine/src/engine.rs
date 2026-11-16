use crate::doc::*;
use crate::fx::Fx;
use crate::presets;
use crate::synth::{self, gm_pad, BassVoice, Voice};

const VOICES: usize = 24;

#[derive(Clone, Copy)]
enum EvKind {
    Drum { pad: u8, vel: f32 },
    BassOn { note: u8, vel: f32 },
    BassOff { note: u8 },
}

struct Ev {
    beat: f64,
    kind: EvKind,
}

struct Schedule {
    events: Vec<Ev>,
    len: f64,
}

#[derive(Clone, Copy, PartialEq)]
enum SchedRef {
    Pattern(usize),
    Song(usize),
}

struct Layer {
    source: SchedRef,
    looping: bool,
    origin: f64,
    cursor: usize,
}

struct Oneshot {
    pattern: usize,
    origin: f64,
    until: f64,
    then_latch: Option<usize>,
    stop_after: bool,
}

struct Pending {
    at: f64,
    pad: u8,
}

struct Click {
    env: f32,
    phase: f32,
    freq: f32,
}

/// Copyable snapshot for a plugin UI. No heap use while filling it.
#[derive(Clone)]
pub struct View {
    pub mode: u8,
    pub playing: bool,
    pub recording: bool,
    pub tempo: f32,
    pub key: i8,
    pub swing: f32,
    pub bass_mute: bool,
    pub metronome: bool,
    pub drum_part: bool,
    pub pattern: u16,
    pub song: u16,
    pub kit: u16,
    pub bass: u16,
    pub bar: u16,
    pub step: u16,
    pub steps: u16,
    pub peak: f32,
    pub names: [[u8; 24]; 4],
    pub song_pads: [[u8; 10]; 6],
    pub grid: [[u8; 32]; 13],
    pub bass_grid: [i16; 32],
}

impl View {
    pub fn new() -> Self {
        Self {
            mode: 1,
            playing: false,
            recording: false,
            tempo: 104.0,
            key: 9,
            swing: 0.0,
            bass_mute: false,
            metronome: false,
            drum_part: true,
            pattern: 0,
            song: 0,
            kit: 0,
            bass: 0,
            bar: 1,
            step: 0,
            steps: 16,
            peak: 0.0,
            names: [[0; 24]; 4],
            song_pads: [[0; 10]; 6],
            grid: [[0; 32]; 13],
            bass_grid: [-1; 32],
        }
    }
}

pub struct Engine {
    sr: f32,
    doc: Document,
    mode: u8,
    playing: bool,
    suspended: bool,
    host_was_playing: bool,
    recording: bool,
    beat: f64,
    prev_beat: f64,
    primed: bool,
    tempo: f32,
    key: i8,
    octave: i8,
    bass_mute: bool,
    metronome: bool,
    drum_part: bool,
    quantize: u8,
    pattern_ix: usize,
    song_ix: usize,
    kit_ix: usize,
    bass_ix: usize,
    kit_locked: bool,
    jam: bool,
    latched: usize,
    oneshot: Option<Oneshot>,
    pending: Option<Pending>,
    pattern_sched: Vec<Schedule>,
    song_sched: Vec<Schedule>,
    layers: Vec<Layer>,
    voices: Vec<Voice>,
    bass_voice: BassVoice,
    fx: Fx,
    click: Click,
    seed: u32,
    peak: f32,
    heard_pattern: usize,
    heard_local: f64,
    dirty: bool,
    mix: Vec<f32>,
    held_bass: Option<(u8, f64)>,
}

impl Engine {
    pub fn new(sr: f32) -> Self {
        let doc = presets::factory();
        let song = &doc.songs[0];
        let latched = song.pads[1].pattern;
        let pat = &doc.patterns[latched];
        let mut engine = Self {
            sr: sr.max(8_000.0),
            tempo: song.bpm,
            key: song.key,
            kit_ix: pat.kit,
            bass_ix: pat.bass,
            latched,
            heard_pattern: latched,
            pattern_ix: latched,
            doc,
            mode: 1,
            playing: false,
            suspended: false,
            host_was_playing: false,
            recording: false,
            beat: 0.0,
            prev_beat: 0.0,
            primed: false,
            octave: 0,
            bass_mute: false,
            metronome: false,
            drum_part: true,
            quantize: 16,
            song_ix: 0,
            kit_locked: false,
            jam: false,
            oneshot: None,
            pending: None,
            pattern_sched: Vec::new(),
            song_sched: Vec::new(),
            layers: Vec::new(),
            voices: (0..VOICES).map(|_| Voice::silent()).collect(),
            bass_voice: BassVoice::new(),
            fx: Fx::new(sr),
            click: Click { env: 0.0, phase: 0.0, freq: 1200.0 },
            seed: 1,
            peak: 0.0,
            heard_local: 0.0,
            dirty: false,
            mix: vec![0.0; 8192 * 2],
            held_bass: None,
        };
        engine.rebuild_schedules();
        engine.layers_for_transport();
        engine
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sr = sr.max(8_000.0);
        self.fx.resize(self.sr);
    }

    pub fn take_dirty(&mut self) -> bool {
        let d = self.dirty;
        self.dirty = false;
        d
    }

    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    pub fn export_json(&self) -> String {
        serde_json::to_string(&self.doc).unwrap_or_else(|_| "{}".into())
    }

    pub fn load_json(&mut self, json: &str) -> bool {
        let Ok(doc) = serde_json::from_str::<Document>(json) else {
            return false;
        };
        if doc.patterns.is_empty() || doc.songs.is_empty() || doc.kits.is_empty() || doc.basses.is_empty() {
            return false;
        }
        self.doc = doc;
        self.clamp_indices();
        self.rebuild_schedules();
        self.layers_for_transport();
        self.dirty = false;
        true
    }

    pub fn command_json(&mut self, json: &str) -> Result<(), String> {
        let cmd: Command = serde_json::from_str(json).map_err(|e| e.to_string())?;
        self.command(cmd);
        Ok(())
    }

    pub fn command(&mut self, cmd: Command) {
        match cmd {
            Command::Play => self.play(),
            Command::Stop => self.stop(),
            Command::ToggleRec => self.recording = !self.recording,
            Command::SetMode { mode } => {
                self.mode = mode.min(2);
                self.jam = false;
                self.oneshot = None;
                self.pending = None;
                self.layers_for_transport();
            }
            Command::SetPart { drum } => self.drum_part = drum,
            Command::SetLiveDrums { .. } => {}
            Command::SetPattern { index } => self.select_pattern(index),
            Command::SetSong { index } => self.select_song(index),
            Command::SetTempo { bpm } => {
                self.tempo = bpm.clamp(40.0, 280.0);
                if self.mode == 0 && self.pattern_ix < self.doc.patterns.len() {
                    self.doc.patterns[self.pattern_ix].bpm = self.tempo;
                    self.dirty = true;
                }
            }
            Command::SetSwing { amount } => {
                let amount = amount.clamp(0.0, 0.72);
                let ix = self.edit_ix();
                if ix < self.doc.patterns.len() {
                    self.doc.patterns[ix].swing = amount;
                    self.rebuild_schedules();
                    self.dirty = true;
                }
            }
            Command::SetKey { key } => {
                self.key = key.rem_euclid(12);
                if self.song_ix < self.doc.songs.len() {
                    self.doc.songs[self.song_ix].key = self.key;
                }
                self.dirty = true;
            }
            Command::SetKit { index } => {
                if index < self.doc.kits.len() {
                    self.kit_ix = index;
                    self.kit_locked = true;
                    self.dirty = true;
                }
            }
            Command::SetBass { index } => {
                if index < self.doc.basses.len() {
                    self.bass_ix = index;
                    self.dirty = true;
                }
            }
            Command::SetBassMute { mute } => {
                self.bass_mute = mute;
                if mute {
                    self.bass_voice.release();
                }
            }
            Command::SetMetronome { on } => self.metronome = on,
            Command::SetQuantize { division } => self.quantize = division,
            Command::SetOctave { octave } => self.octave = octave.clamp(-2, 2),
            Command::PadDown { pad, vel } => self.pad_down(pad, vel),
            Command::PadUp { pad } => self.pad_up(pad),
            Command::Section { pad } => self.section(pad),
            Command::Groove { pad } => self.groove(pad),
            Command::ToggleStep { pad, step, vel } => self.toggle_step(pad, step, vel),
            Command::SetBassStep { step, note, dur } => self.set_bass_step(step, note, dur),
            Command::ClearDrums => {
                let ix = self.edit_ix();
                if ix < self.doc.patterns.len() {
                    self.doc.patterns[ix].drums.clear();
                    self.rebuild_schedules();
                    self.dirty = true;
                }
            }
            Command::ClearBass => {
                let ix = self.edit_ix();
                if ix < self.doc.patterns.len() {
                    self.doc.patterns[ix].bassline.clear();
                    self.rebuild_schedules();
                    self.dirty = true;
                }
            }
            Command::DuplicatePattern => self.duplicate_pattern(),
            Command::SetFx { fx } => {
                self.doc.fx = fx;
                self.dirty = true;
            }
            Command::Midi { channel, note, vel, on } => self.midi(channel, note, vel, on),
            Command::LoadDocument { json } => {
                self.load_json(&json);
            }
            Command::ResetFactory => {
                *self = Self::new(self.sr);
            }
            Command::Panic => {
                for v in &mut self.voices {
                    v.on = false;
                }
                self.bass_voice.release();
                self.bass_voice.active = false;
            }
        }
    }

    pub fn render(
        &mut self,
        frames: usize,
        host: Option<HostPos>,
        midi: &[MidiEvt],
        out_l: &mut [f32],
        out_r: &mut [f32],
    ) {
        let mut midi_i = 0;
        for i in 0..frames {
            while midi_i < midi.len() && midi[midi_i].frame as usize <= i {
                let e = midi[midi_i];
                self.midi(e.channel, e.note, e.vel, e.on);
                midi_i += 1;
            }
            let sample_host = host.map(|h| {
                let bps = h.tempo as f64 / 60.0;
                HostPos {
                    playing: h.playing,
                    tempo: h.tempo,
                    beat: h.beat + i as f64 * bps / self.sr as f64,
                }
            });
            let (l, r) = self.process_sample(sample_host);
            if i < out_l.len() {
                out_l[i] = l;
            }
            if i < out_r.len() {
                out_r[i] = r;
            }
        }
    }

    pub fn render_interleaved(&mut self, frames: usize) -> *const f32 {
        let frames = frames.min(4096);
        for i in 0..frames {
            let (l, r) = self.process_sample(None);
            self.mix[i * 2] = l;
            self.mix[i * 2 + 1] = r;
        }
        self.mix.as_ptr()
    }

    pub fn hud_json(&self) -> String {
        let p = self.shown_pattern();
        let pattern = self.doc.patterns.get(p);
        let steps = pattern.map(|p| p.steps as usize).unwrap_or(16).min(64);
        let mut grid = vec![vec![0u8; steps]; PADS];
        let mut bass_grid = vec![-1i16; steps];
        if let Some(pattern) = pattern {
            for hit in &pattern.drums {
                let step = (hit.tick / 24) as usize;
                if step < steps && (hit.pad as usize) < PADS {
                    grid[hit.pad as usize][step] = grid[hit.pad as usize][step].max((hit.vel * 127.0) as u8);
                }
            }
            for note in &pattern.bassline {
                let step = (note.tick / 24) as usize;
                if step < steps {
                    bass_grid[step] = note.note as i16 + self.key as i16;
                }
            }
        }
        let song = self.doc.songs.get(self.song_ix);
        let pads: Vec<String> = song
            .map(|s| s.pads.iter().take(6).map(|p| p.label.clone()).collect())
            .unwrap_or_default();
        let step_pos = if steps == 0 { 0 } else { (self.heard_local / 0.25).floor().max(0.0) as usize % steps };
        serde_json::json!({
            "mode": self.mode,
            "playing": self.transport_running(),
            "recording": self.recording,
            "tempo": self.tempo,
            "key": self.key,
            "keyName": KEY_NAMES[self.key.rem_euclid(12) as usize],
            "swing": pattern.map(|p| p.swing).unwrap_or(0.0),
            "bassMute": self.bass_mute,
            "metronome": self.metronome,
            "drumPart": self.drum_part,
            "octave": self.octave,
            "quantize": self.quantize,
            "pattern": p,
            "patternName": pattern.map(|p| p.name.as_str()).unwrap_or(""),
            "song": self.song_ix,
            "songName": song.map(|s| s.name.as_str()).unwrap_or(""),
            "kit": self.kit_ix,
            "kitName": self.doc.kits.get(self.kit_ix).map(|k| k.name.as_str()).unwrap_or(""),
            "bass": self.bass_ix,
            "bassName": self.doc.basses.get(self.bass_ix).map(|b| b.name.as_str()).unwrap_or(""),
            "bar": (self.heard_local / 4.0).floor().max(0.0) as u32 + 1,
            "step": step_pos,
            "steps": steps,
            "peak": self.peak,
            "grid": grid,
            "bassGrid": bass_grid,
            "songPads": pads,
            "fx": self.doc.fx,
            "patternCount": self.doc.patterns.len(),
            "songCount": self.doc.songs.len(),
        })
        .to_string()
    }

    pub fn catalog_json(&self) -> String {
        serde_json::json!({
            "songs": self.doc.songs.iter().map(|s| &s.name).collect::<Vec<_>>(),
            "patterns": self.doc.patterns.iter().map(|p| &p.name).collect::<Vec<_>>(),
            "kits": self.doc.kits.iter().map(|k| &k.name).collect::<Vec<_>>(),
            "basses": self.doc.basses.iter().map(|b| &b.name).collect::<Vec<_>>(),
            "pads": PAD_NAMES,
            "keys": KEY_NAMES,
        })
        .to_string()
    }

    pub fn write_view(&self, view: &mut View) {
        let p = self.shown_pattern();
        view.mode = self.mode;
        view.playing = self.transport_running();
        view.recording = self.recording;
        view.tempo = self.tempo;
        view.key = self.key;
        view.bass_mute = self.bass_mute;
        view.metronome = self.metronome;
        view.drum_part = self.drum_part;
        view.pattern = p as u16;
        view.song = self.song_ix as u16;
        view.kit = self.kit_ix as u16;
        view.bass = self.bass_ix as u16;
        view.peak = self.peak;
        view.swing = self.doc.patterns.get(p).map(|p| p.swing).unwrap_or(0.0);
        let steps = self.doc.patterns.get(p).map(|p| p.steps).unwrap_or(16).min(32);
        view.steps = steps;
        view.step = if steps == 0 { 0 } else { (self.heard_local / 0.25).floor().max(0.0) as u16 % steps };
        view.bar = (self.heard_local / 4.0).floor().max(0.0) as u16 + 1;
        write_name(&mut view.names[0], self.doc.patterns.get(p).map(|p| p.name.as_str()).unwrap_or(""));
        write_name(&mut view.names[1], self.doc.songs.get(self.song_ix).map(|s| s.name.as_str()).unwrap_or(""));
        write_name(&mut view.names[2], self.doc.kits.get(self.kit_ix).map(|k| k.name.as_str()).unwrap_or(""));
        write_name(&mut view.names[3], self.doc.basses.get(self.bass_ix).map(|b| b.name.as_str()).unwrap_or(""));
        for row in &mut view.song_pads {
            row.fill(0);
        }
        if let Some(song) = self.doc.songs.get(self.song_ix) {
            for (i, pad) in song.pads.iter().take(6).enumerate() {
                write_name_n(&mut view.song_pads[i], &pad.label, 10);
            }
        }
        view.grid = [[0; 32]; 13];
        view.bass_grid = [-1; 32];
        if let Some(pattern) = self.doc.patterns.get(p) {
            for hit in &pattern.drums {
                let step = (hit.tick / 24) as usize;
                if step < 32 && (hit.pad as usize) < 13 {
                    view.grid[hit.pad as usize][step] =
                        view.grid[hit.pad as usize][step].max((hit.vel * 127.0) as u8);
                }
            }
            for note in &pattern.bassline {
                let step = (note.tick / 24) as usize;
                if step < 32 {
                    view.bass_grid[step] = note.note as i16 + self.key as i16;
                }
            }
        }
    }

    pub fn process_sample(&mut self, host: Option<HostPos>) -> (f32, f32) {
        let prev = self.prev_beat;
        let running = self.advance(host);
        if running {
            let newb = self.beat;
            if self.primed {
                self.primed = false;
                self.resolve(prev, newb);
                self.fire_primed(newb);
            } else if newb + 1e-6 < prev || newb - prev > 1.0 {
                self.beat = newb;
                self.resync_layers();
            } else {
                self.resolve(prev, newb);
                self.fire_all(prev, newb);
                self.metro(prev, newb);
            }
            self.prev_beat = self.beat;
        }

        let mut drum_l = 0.0;
        let mut drum_r = 0.0;
        let mut send_l = 0.0;
        let mut send_r = 0.0;
        for voice in &mut self.voices {
            if !voice.on {
                continue;
            }
            let s = synth::render_voice(voice, self.sr);
            let (l, r) = pan(s, voice.pan);
            drum_l += l;
            drum_r += r;
            send_l += l * voice.send;
            send_r += r * voice.send;
        }
        let bass = if self.bass_mute {
            0.0
        } else if self.bass_ix < self.doc.basses.len() {
            let prog = self.doc.basses[self.bass_ix].clone();
            self.bass_voice.render(&prog, self.sr)
        } else {
            0.0
        };
        let (bl, br) = (bass * 0.5, bass * 0.5);
        send_l += bl * 0.25;
        send_r += br * 0.25;
        let mut l = drum_l * 0.62 + bl * 0.85;
        let mut r = drum_r * 0.62 + br * 0.85;
        if self.click.env > 0.001 {
            self.click.phase += self.click.freq / self.sr;
            let c = (self.click.phase * std::f32::consts::TAU).sin() * self.click.env;
            self.click.env *= 0.992;
            l += c;
            r += c;
        }
        let fx = self.doc.fx;
        let (l, r) = self.fx.process(l, r, send_l, send_r, &fx, self.tempo);
        self.peak = (self.peak * 0.999).max(l.abs()).max(r.abs());
        (l, r)
    }

    fn advance(&mut self, host: Option<HostPos>) -> bool {
        if let Some(h) = host {
            if h.playing {
                let started = !self.host_was_playing;
                self.host_was_playing = true;
                if started {
                    self.suspended = false;
                    self.layers_for_transport();
                    if h.beat < 0.04 {
                        self.primed = true;
                        self.prev_beat = 0.0;
                    } else {
                        self.prev_beat = h.beat;
                        self.resync_layers();
                    }
                }
                if self.suspended {
                    return false;
                }
                self.tempo = h.tempo.clamp(40.0, 280.0);
                self.beat = h.beat.max(0.0);
                return true;
            } else if self.host_was_playing {
                self.host_was_playing = false;
                self.suspended = false;
                self.playing = false;
                self.bass_voice.release();
                return false;
            }
        }
        if self.playing && !self.suspended {
            self.beat += (self.tempo as f64 / 60.0) / self.sr as f64;
            return true;
        }
        false
    }

    fn play(&mut self) {
        self.suspended = false;
        self.playing = true;
        if !self.host_was_playing {
            self.beat = 0.0;
            self.prev_beat = 0.0;
            self.primed = true;
            self.jam = false;
            self.oneshot = None;
            self.pending = None;
            self.layers_for_transport();
            for layer in &mut self.layers {
                layer.cursor = 0;
            }
        }
    }

    fn stop(&mut self) {
        self.playing = false;
        self.suspended = true;
        self.recording = false;
        self.pending = None;
        self.oneshot = None;
        self.jam = false;
        if !self.host_was_playing {
            self.beat = 0.0;
            self.prev_beat = 0.0;
        }
        self.bass_voice.release();
        self.layers_for_transport();
    }

    fn transport_running(&self) -> bool {
        (self.playing || self.host_was_playing) && !self.suspended
    }

    fn resolve(&mut self, _prev: f64, newb: f64) {
        if let Some(pending) = &self.pending {
            if newb + 1e-6 >= pending.at {
                let pad = pending.pad;
                self.pending = None;
                self.apply_section(pad);
            }
        }
        if let Some(one) = &self.oneshot {
            if newb + 1e-6 >= one.until {
                let then = one.then_latch;
                let stop = one.stop_after;
                self.oneshot = None;
                self.bass_voice.release();
                if stop {
                    self.playing = false;
                    self.suspended = true;
                    self.jam = false;
                } else if let Some(p) = then {
                    self.latched = p;
                    self.jam = true;
                    self.recall_pattern(p, false);
                }
                self.layers_for_transport();
            }
        }
    }

    fn fire_primed(&mut self, newb: f64) {
        let n = self.layers.len();
        for i in 0..n {
            self.layers[i].cursor = 0;
            let local = self.layer_local(i, newb);
            self.fire_range(i, -0.0001, local.max(0.0));
            self.mark_heard(i, newb);
        }
        self.metro(0.0, newb.max(0.001));
    }

    fn fire_all(&mut self, prev: f64, newb: f64) {
        let n = self.layers.len();
        for i in 0..n {
            self.fire_layer(i, prev, newb);
        }
    }

    fn fire_layer(&mut self, i: usize, prev: f64, newb: f64) {
        let (looping, origin) = {
            let layer = &self.layers[i];
            (layer.looping, layer.origin)
        };
        let len = self.layer_len(i).max(0.25);
        let (prev_l, new_l) = if looping {
            (prev.rem_euclid(len), newb.rem_euclid(len))
        } else {
            ((prev - origin).max(-0.001), newb - origin)
        };
        if looping && new_l + 1e-8 < prev_l {
            self.fire_range(i, prev_l, len + 1.0);
            self.layers[i].cursor = 0;
            self.fire_range(i, -0.0001, new_l);
        } else {
            self.fire_range(i, prev_l, new_l);
        }
        self.mark_heard(i, newb);
    }

    fn fire_range(&mut self, layer: usize, from: f64, to: f64) {
        let source = self.layers[layer].source;
        let mut c = self.layers[layer].cursor;
        loop {
            let next = {
                let Some(sched) = self.schedule(source) else { break };
                if c >= sched.events.len() {
                    break;
                }
                let ev = &sched.events[c];
                if ev.beat > to + 1e-9 {
                    break;
                }
                if ev.beat <= from + 1e-9 {
                    None
                } else {
                    Some(ev.kind)
                }
            };
            c += 1;
            if let Some(kind) = next {
                self.trigger(kind);
            } else if c > 10_000 {
                break;
            } else {
                // skipped an event before `from`, or we need to check the bound again
                let Some(sched) = self.schedule(source) else { break };
                if c > sched.events.len() {
                    break;
                }
                if c > 0 && c <= sched.events.len() && sched.events[c - 1].beat > to + 1e-9 {
                    c -= 1;
                    break;
                }
            }
        }
        if layer < self.layers.len() {
            self.layers[layer].cursor = c;
        }
    }

    fn trigger(&mut self, kind: EvKind) {
        match kind {
            EvKind::Drum { pad, vel } => self.hit_drum(pad, vel, false),
            EvKind::BassOn { note, vel } => {
                if !self.bass_mute {
                    let sounding = (note as i16 + self.key as i16).clamp(12, 96) as u8;
                    self.bass_on(sounding, vel);
                }
            }
            EvKind::BassOff { note } => {
                let _ = note;
                self.bass_voice.release();
            }
        }
    }

    fn metro(&mut self, prev: f64, newb: f64) {
        if !self.metronome {
            return;
        }
        let a = prev.floor() as i64;
        let b = newb.floor() as i64;
        if b != a && b >= 0 {
            let down = b.rem_euclid(4) == 0;
            self.click.env = if down { 0.28 } else { 0.14 };
            self.click.phase = 0.0;
            self.click.freq = if down { 1900.0 } else { 1200.0 };
        }
    }

    fn pad_down(&mut self, pad: u8, vel: f32) {
        if pad as usize >= PADS {
            return;
        }
        if self.drum_part {
            self.hit_drum(pad, vel, true);
        } else {
            let note = self.bass_note_for_pad(pad);
            self.bass_on(note, vel);
            if self.recording && self.mode == 0 {
                self.held_bass = Some((note, self.record_beat()));
            }
        }
    }

    fn pad_up(&mut self, pad: u8) {
        if self.drum_part {
            return;
        }
        let note = self.bass_note_for_pad(pad);
        if self.recording && self.mode == 0 {
            if let Some((held, start)) = self.held_bass.take() {
                if held == note {
                    self.record_bass(held, start, self.record_beat());
                }
            }
        }
        self.bass_voice.note_off(note);
    }

    fn section(&mut self, pad: u8) {
        if self.song_ix >= self.doc.songs.len() || pad as usize >= PADS {
            return;
        }
        if !self.transport_running() {
            self.beat = 0.0;
            self.prev_beat = 0.0;
            self.primed = true;
            self.playing = true;
            self.suspended = false;
            self.apply_section(pad);
            return;
        }
        let job = self.doc.songs[self.song_ix].pads[pad as usize].job;
        let grid = match job {
            PadJob::Fill => 1.0,
            PadJob::Empty => return,
            _ => {
                let pattern = self.doc.songs[self.song_ix].pads[pad as usize].pattern;
                self.pattern_len(pattern).max(4.0)
            }
        };
        let at = quantize_up(self.beat, grid);
        if (at - self.beat).abs() < 0.03 {
            self.apply_section(pad);
        } else {
            self.pending = Some(Pending { at, pad });
        }
    }

    fn apply_section(&mut self, pad: u8) {
        let Some(song_pad) = self.doc.songs.get(self.song_ix).and_then(|s| s.pads.get(pad as usize)).cloned() else {
            return;
        };
        self.mode = 1;
        match song_pad.job {
            PadJob::Empty => return,
            PadJob::Loop => {
                self.jam = true;
                self.oneshot = None;
                self.latched = song_pad.pattern;
                self.recall_pattern(song_pad.pattern, false);
            }
            PadJob::OnceThen { goto } => {
                let len = self.pattern_len(song_pad.pattern);
                let then = self.doc.songs[self.song_ix].pads.get(goto as usize).map(|p| p.pattern);
                self.jam = true;
                self.oneshot = Some(Oneshot {
                    pattern: song_pad.pattern,
                    origin: self.beat,
                    until: self.beat + len,
                    then_latch: then,
                    stop_after: false,
                });
                self.recall_pattern(song_pad.pattern, false);
            }
            PadJob::Fill => {
                let len = self.pattern_len(song_pad.pattern);
                self.jam = true;
                self.oneshot = Some(Oneshot {
                    pattern: song_pad.pattern,
                    origin: self.beat,
                    until: self.beat + len,
                    then_latch: Some(self.latched),
                    stop_after: false,
                });
            }
            PadJob::Stop => {
                let len = self.pattern_len(song_pad.pattern);
                self.jam = true;
                self.oneshot = Some(Oneshot {
                    pattern: song_pad.pattern,
                    origin: self.beat,
                    until: self.beat + len,
                    then_latch: None,
                    stop_after: true,
                });
            }
        }
        self.bass_voice.release();
        self.layers_for_transport();
    }

    fn groove(&mut self, pad: u8) {
        if self.song_ix >= self.doc.songs.len() || pad as usize >= PADS {
            return;
        }
        let song_pad = self.doc.songs[self.song_ix].pads[pad as usize].clone();
        if song_pad.job == PadJob::Empty {
            return;
        }
        self.mode = 2;
        self.oneshot = None;
        self.jam = false;
        if let Some(pos) = self.layers.iter().position(|l| l.source == SchedRef::Pattern(song_pad.pattern)) {
            self.layers.remove(pos);
        } else if self.layers.len() < 4 {
            let mut layer = self.looping(SchedRef::Pattern(song_pad.pattern));
            layer.cursor = self.cursor_for(layer.source, self.beat);
            self.layers.push(layer);
        }
        if self.layers.is_empty() {
            self.layers.push(self.looping(SchedRef::Pattern(self.latched)));
        }
        if !self.transport_running() {
            self.playing = true;
            self.suspended = false;
            self.primed = true;
            self.beat = 0.0;
            self.prev_beat = 0.0;
        }
    }

    fn select_pattern(&mut self, index: usize) {
        if index >= self.doc.patterns.len() {
            return;
        }
        self.mode = 0;
        self.pattern_ix = index;
        self.jam = false;
        self.oneshot = None;
        self.recall_pattern(index, true);
        self.layers_for_transport();
    }

    fn select_song(&mut self, index: usize) {
        if index >= self.doc.songs.len() {
            return;
        }
        self.song_ix = index;
        self.mode = 1;
        self.jam = false;
        self.oneshot = None;
        self.pending = None;
        self.kit_locked = false;
        let song = &self.doc.songs[index];
        self.tempo = song.bpm;
        self.key = song.key;
        self.latched = song.pads.get(1).map(|p| p.pattern).unwrap_or(0);
        let latched = self.latched;
        self.recall_pattern(latched, false);
        self.layers_for_transport();
        self.dirty = true;
    }

    fn recall_pattern(&mut self, index: usize, set_tempo: bool) {
        let Some(pattern) = self.doc.patterns.get(index) else { return };
        self.pattern_ix = index;
        self.heard_pattern = index;
        if !self.kit_locked {
            self.kit_ix = pattern.kit.min(self.doc.kits.len().saturating_sub(1));
        }
        self.bass_ix = pattern.bass.min(self.doc.basses.len().saturating_sub(1));
        self.key = pattern.key;
        if set_tempo && !self.host_was_playing {
            self.tempo = pattern.bpm;
        }
    }

    fn hit_drum(&mut self, pad: u8, vel: f32, from_user: bool) {
        if pad as usize >= PADS || self.kit_ix >= self.doc.kits.len() {
            return;
        }
        let assign = self.doc.kits[self.kit_ix].pads.get(pad as usize).cloned();
        let Some(assign) = assign else { return };
        if from_user && self.recording && self.mode == 0 {
            self.record_drum(pad, vel);
        }
        self.seed = self.seed.wrapping_add(1);
        let idx = self.alloc_voice();
        synth::trigger(&mut self.voices[idx], assign.inst, vel, &assign, self.seed);
    }

    fn bass_on(&mut self, note: u8, vel: f32) {
        self.seed = self.seed.wrapping_add(1);
        self.bass_voice.note_on(note, vel, self.bass_ix, self.seed);
    }

    fn record_drum(&mut self, pad: u8, vel: f32) {
        let ix = self.pattern_ix;
        let beat = self.quantized_local(ix);
        let tick = (beat * 96.0).round() as u32;
        let pattern = &mut self.doc.patterns[ix];
        pattern.drums.retain(|h| !(h.pad == pad && h.tick.abs_diff(tick) < 12));
        pattern.drums.push(DrumHit { tick, pad, vel });
        self.rebuild_schedules();
        self.dirty = true;
    }

    fn record_bass(&mut self, sounding: u8, start: f64, end: f64) {
        let ix = self.pattern_ix;
        let len = self.pattern_len(ix).max(0.25);
        let mut dur_beats = (end - start).rem_euclid(len).max(0.1);
        if dur_beats > len {
            dur_beats = 0.25;
        }
        let note = (sounding as i16 - self.key as i16).clamp(0, 96) as u8;
        let tick = (start.rem_euclid(len) * 96.0).round() as u32;
        let dur = (dur_beats * 96.0).round().max(12.0) as u32;
        self.doc.patterns[ix].bassline.retain(|n| n.tick.abs_diff(tick) > 12);
        self.doc.patterns[ix].bassline.push(BassNote { tick, note, dur, vel: 0.9 });
        self.rebuild_schedules();
        self.dirty = true;
    }

    fn toggle_step(&mut self, pad: u8, step: usize, vel: u8) {
        let ix = self.edit_ix();
        if ix >= self.doc.patterns.len() || pad as usize >= PADS {
            return;
        }
        let tick = (step as u32) * 24;
        let pattern = &mut self.doc.patterns[ix];
        if pattern.drums.iter().any(|h| h.pad == pad && h.tick / 24 == step as u32) {
            pattern.drums.retain(|h| !(h.pad == pad && h.tick / 24 == step as u32));
        } else {
            let vel = if vel == 0 { 0.8 } else { vel as f32 / 127.0 };
            pattern.drums.push(DrumHit { tick, pad, vel });
        }
        self.rebuild_schedules();
        self.dirty = true;
    }

    fn set_bass_step(&mut self, step: usize, note: i16, dur: u8) {
        let ix = self.edit_ix();
        if ix >= self.doc.patterns.len() {
            return;
        }
        let tick = (step as u32) * 24;
        let pattern = &mut self.doc.patterns[ix];
        pattern.bassline.retain(|n| n.tick / 24 != step as u32);
        if note >= 0 {
            let stored = (note - self.key as i16).clamp(0, 96) as u8;
            let dur = (dur.max(1) as u32) * 24;
            pattern.bassline.push(BassNote { tick, note: stored, dur, vel: 0.9 });
        }
        self.rebuild_schedules();
        self.dirty = true;
    }

    fn duplicate_pattern(&mut self) {
        let ix = self.edit_ix();
        if ix >= self.doc.patterns.len() {
            return;
        }
        let mut copy = self.doc.patterns[ix].clone();
        copy.name = format!("{} copy", copy.name);
        self.doc.patterns.push(copy);
        self.pattern_ix = self.doc.patterns.len() - 1;
        self.mode = 0;
        self.rebuild_schedules();
        self.layers_for_transport();
        self.dirty = true;
    }

    pub fn midi(&mut self, channel: u8, note: u8, vel: f32, on: bool) {
        if channel == 9 {
            if on {
                if let Some(pad) = gm_pad(note) {
                    self.hit_drum(pad, vel.max(0.05), true);
                }
            }
            return;
        }
        if on {
            self.bass_on(note, vel.max(0.05));
            if self.recording && self.mode == 0 {
                self.held_bass = Some((note, self.record_beat()));
            }
        } else {
            if self.recording && self.mode == 0 {
                if let Some((held, start)) = self.held_bass.take() {
                    if held == note {
                        self.record_bass(held, start, self.record_beat());
                    }
                }
            }
            self.bass_voice.note_off(note);
        }
    }

    fn record_beat(&self) -> f64 {
        self.quantized_local(self.pattern_ix)
    }

    fn quantized_local(&self, pattern: usize) -> f64 {
        let len = self.pattern_len(pattern).max(0.25);
        let local = if self.transport_running() {
            self.beat.rem_euclid(len)
        } else {
            0.0
        };
        let q = match self.quantize {
            4 => 1.0,
            8 => 0.5,
            16 => 0.25,
            32 => 0.125,
            _ => 0.0,
        };
        if q <= 0.0 {
            local
        } else {
            (local / q).round() * q
        }
        .rem_euclid(len)
    }

    fn edit_ix(&self) -> usize {
        if self.mode == 0 { self.pattern_ix } else { self.shown_pattern() }
    }

    fn shown_pattern(&self) -> usize {
        if self.mode == 0 {
            self.pattern_ix
        } else if self.jam {
            self.oneshot.as_ref().map(|o| o.pattern).unwrap_or(self.latched)
        } else {
            self.heard_pattern
        }
    }

    fn layers_for_transport(&mut self) {
        self.layers.clear();
        if self.mode == 2 {
            self.layers.push(self.looping(SchedRef::Pattern(self.latched)));
        } else if self.mode == 0 {
            self.layers.push(self.looping(SchedRef::Pattern(self.pattern_ix)));
        } else if let Some(one) = &self.oneshot {
            self.layers.push(Layer {
                source: SchedRef::Pattern(one.pattern),
                looping: false,
                origin: one.origin,
                cursor: 0,
            });
        } else if self.jam {
            self.layers.push(self.looping(SchedRef::Pattern(self.latched)));
        } else if self.song_ix < self.song_sched.len() {
            self.layers.push(self.looping(SchedRef::Song(self.song_ix)));
        }
        self.resync_layers();
    }

    fn looping(&self, source: SchedRef) -> Layer {
        Layer { source, looping: true, origin: 0.0, cursor: 0 }
    }

    fn resync_layers(&mut self) {
        let beat = self.beat;
        for i in 0..self.layers.len() {
            if self.beat < 0.02 {
                self.layers[i].cursor = 0;
            } else {
                let source = self.layers[i].source;
                self.layers[i].cursor = self.cursor_for(source, beat);
            }
        }
    }

    fn cursor_for(&self, source: SchedRef, beat: f64) -> usize {
        let Some(sched) = self.schedule(source) else { return 0 };
        let len = sched.len.max(0.25);
        let local = beat.rem_euclid(len);
        sched.events.partition_point(|e| e.beat <= local)
    }

    fn mark_heard(&mut self, layer: usize, beat: f64) {
        let source = self.layers[layer].source;
        match source {
            SchedRef::Pattern(p) => {
                self.heard_pattern = p;
                let len = self.pattern_len(p).max(0.25);
                if self.layers[layer].looping {
                    self.heard_local = beat.rem_euclid(len);
                } else {
                    self.heard_local = (beat - self.layers[layer].origin).clamp(0.0, len);
                }
            }
            SchedRef::Song(s) => {
                let (p, local) = self.locate_song(s, beat);
                self.heard_pattern = p;
                self.heard_local = local;
            }
        }
    }

    fn locate_song(&self, song_ix: usize, beat: f64) -> (usize, f64) {
        let Some(song) = self.doc.songs.get(song_ix) else { return (0, 0.0) };
        let total = self.song_sched.get(song_ix).map(|s| s.len).unwrap_or(4.0).max(0.25);
        let mut b = beat.rem_euclid(total);
        for step in &song.arrangement {
            let plen = self.pattern_len(step.pattern).max(0.25);
            let span = plen * step.repeats as f64;
            if b < span {
                return (step.pattern, b % plen);
            }
            b -= span;
        }
        (self.latched, 0.0)
    }

    fn layer_local(&self, i: usize, beat: f64) -> f64 {
        let len = self.layer_len(i).max(0.25);
        if self.layers[i].looping {
            beat.rem_euclid(len)
        } else {
            beat - self.layers[i].origin
        }
    }

    fn layer_len(&self, i: usize) -> f64 {
        let source = self.layers[i].source;
        self.schedule(source).map(|s| s.len).unwrap_or(4.0)
    }

    fn pattern_len(&self, index: usize) -> f64 {
        self.pattern_sched.get(index).map(|s| s.len).unwrap_or(4.0)
    }

    fn schedule(&self, source: SchedRef) -> Option<&Schedule> {
        match source {
            SchedRef::Pattern(i) => self.pattern_sched.get(i),
            SchedRef::Song(i) => self.song_sched.get(i),
        }
    }

    fn rebuild_schedules(&mut self) {
        self.pattern_sched = self.doc.patterns.iter().map(build_pattern_sched).collect();
        self.song_sched = self.doc.songs.iter().map(|song| build_song_sched(song, &self.pattern_sched)).collect();
    }

    fn clamp_indices(&mut self) {
        self.pattern_ix = self.pattern_ix.min(self.doc.patterns.len().saturating_sub(1));
        self.song_ix = self.song_ix.min(self.doc.songs.len().saturating_sub(1));
        self.kit_ix = self.kit_ix.min(self.doc.kits.len().saturating_sub(1));
        self.bass_ix = self.bass_ix.min(self.doc.basses.len().saturating_sub(1));
        self.latched = self.latched.min(self.doc.patterns.len().saturating_sub(1));
    }

    fn alloc_voice(&mut self) -> usize {
        if let Some(i) = self.voices.iter().position(|v| !v.on) {
            return i;
        }
        self.voices
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.t.partial_cmp(&b.1.t).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    fn bass_note_for_pad(&self, pad: u8) -> u8 {
        (36 + self.octave as i16 * 12 + self.key as i16 + pad as i16).clamp(12, 96) as u8
    }
}

fn build_pattern_sched(pattern: &Pattern) -> Schedule {
    let mut events = Vec::new();
    for hit in &pattern.drums {
        let beat = swing_beat(hit.tick as f64 / 96.0, pattern.swing);
        events.push(Ev { beat, kind: EvKind::Drum { pad: hit.pad, vel: hit.vel } });
    }
    for note in &pattern.bassline {
        let start = swing_beat(note.tick as f64 / 96.0, pattern.swing);
        let end = note.tick as f64 / 96.0 + note.dur as f64 / 96.0;
        events.push(Ev { beat: start, kind: EvKind::BassOn { note: note.note, vel: note.vel } });
        events.push(Ev { beat: end, kind: EvKind::BassOff { note: note.note } });
    }
    events.sort_by(|a, b| {
        a.beat.partial_cmp(&b.beat).unwrap_or(std::cmp::Ordering::Equal).then_with(|| kind_ord(a.kind).cmp(&kind_ord(b.kind)))
    });
    let steps = pattern.steps.max(1) as f64;
    Schedule { len: steps * 0.25, events }
}

fn build_song_sched(song: &Song, patterns: &[Schedule]) -> Schedule {
    let mut events = Vec::new();
    let mut beat = 0.0;
    for step in &song.arrangement {
        let Some(sched) = patterns.get(step.pattern) else { continue };
        for _ in 0..step.repeats.max(1) {
            for ev in &sched.events {
                events.push(Ev { beat: beat + ev.beat, kind: ev.kind });
            }
            beat += sched.len.max(0.25);
        }
    }
    if beat <= 0.0 {
        beat = 4.0;
    }
    Schedule { events, len: beat }
}

fn kind_ord(kind: EvKind) -> u8 {
    match kind {
        EvKind::BassOff { .. } => 0,
        EvKind::Drum { .. } => 1,
        EvKind::BassOn { .. } => 2,
    }
}

fn swing_beat(beat: f64, swing: f32) -> f64 {
    if swing <= 0.001 {
        return beat;
    }
    let swing = swing.clamp(0.0, 0.72) as f64;
    let step = 0.25;
    let pos = beat.rem_euclid(0.5);
    if pos >= step - 1e-9 {
        let base = beat - pos + step;
        let frac = pos - step;
        let delay = (step - frac) * swing;
        base + frac + delay
    } else {
        beat
    }
}

fn quantize_up(beat: f64, grid: f64) -> f64 {
    if grid <= 0.0 {
        return beat;
    }
    let q = (beat / grid).ceil() * grid;
    if (q - beat).abs() < 1e-4 { beat } else { q }
}

fn pan(sample: f32, pan: f32) -> (f32, f32) {
    let angle = (pan.clamp(-1.0, 1.0) + 1.0) * 0.25 * std::f32::consts::PI;
    (sample * angle.cos(), sample * angle.sin())
}

fn write_name(dest: &mut [u8; 24], text: &str) {
    write_name_n(dest, text, 24);
}

fn write_name_n(dest: &mut [u8], text: &str, _n: usize) {
    dest.fill(0);
    let bytes = text.as_bytes();
    let n = bytes.len().min(dest.len().saturating_sub(1));
    dest[..n].copy_from_slice(&bytes[..n]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn song_makes_finite_sound() {
        let mut engine = Engine::new(44_100.0);
        engine.command(Command::Play);
        let mut l = vec![0.0; 44_100 * 2];
        let mut r = vec![0.0; 44_100 * 2];
        engine.render(l.len(), None, &[], &mut l, &mut r);
        assert!(l.iter().all(|s| s.is_finite()));
        let rms = (l.iter().map(|s| s * s).sum::<f32>() / l.len() as f32).sqrt();
        assert!(rms > 0.02, "rms {rms}");
        let late = (l[44_100..].iter().map(|s| s * s).sum::<f32>() / 44_100.0).sqrt();
        assert!(late > 0.015, "late rms {late}");
    }

    #[test]
    fn pad_hit_is_audible_when_stopped() {
        let mut engine = Engine::new(44_100.0);
        engine.command(Command::PadDown { pad: 0, vel: 1.0 });
        let mut l = vec![0.0; 8_000];
        let mut r = vec![0.0; 8_000];
        engine.render(l.len(), None, &[], &mut l, &mut r);
        let peak = l.iter().fold(0.0_f32, |m, s| m.max(s.abs()));
        assert!(peak > 0.05, "peak {peak}");
    }

    #[test]
    fn host_downbeat_is_finite() {
        let mut engine = Engine::new(48_000.0);
        let mut l = vec![0.0; 24_000];
        let mut r = vec![0.0; 24_000];
        engine.render(l.len(), Some(HostPos { playing: true, tempo: 120.0, beat: 0.0 }), &[], &mut l, &mut r);
        assert!(l.iter().all(|s| s.is_finite()));
        let rms = (l.iter().map(|s| s * s).sum::<f32>() / l.len() as f32).sqrt();
        assert!(rms > 0.02, "rms {rms}");
    }

    #[test]
    fn document_roundtrip() {
        let mut engine = Engine::new(44_100.0);
        let json = engine.export_json();
        assert!(engine.load_json(&json));
        assert!(engine.export_json().len() > 1000);
    }
}
