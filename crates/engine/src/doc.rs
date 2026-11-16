use serde::{Deserialize, Serialize};

pub const PADS: usize = 13;

pub const PAD_NAMES: [&str; PADS] = [
    "KICK", "SNARE", "HAT", "OPEN", "RIM", "COWBELL", "TAMB", "HI TOM", "MID TOM", "LO TOM",
    "RIDE", "CRASH", "CLAP",
];

pub const KEY_NAMES: [&str; 12] = [
    "C", "C#", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B",
];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PadAssign {
    pub inst: u8,
    pub tune: f32,
    pub level: f32,
    pub pan: f32,
    pub decay: f32,
    pub send: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Kit {
    pub name: String,
    pub pads: Vec<PadAssign>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BassProgram {
    pub name: String,
    pub osc: u8,
    pub cutoff: f32,
    pub res: f32,
    pub sustain: f32,
    pub release: f32,
    pub punch: f32,
    pub glide: f32,
    pub sub: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PadJob {
    Empty,
    Loop,
    /// Play once, then latch `goto` as a looping verse.
    OnceThen { goto: u8 },
    /// Play once, then return to the latched verse.
    Fill,
    /// Play once, then stop the band.
    Stop,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DrumHit {
    pub tick: u32,
    pub pad: u8,
    pub vel: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BassNote {
    pub tick: u32,
    /// MIDI note with the song key at C. Playback adds the current key.
    pub note: u8,
    pub dur: u32,
    pub vel: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pattern {
    pub name: String,
    pub kit: usize,
    pub bass: usize,
    pub bpm: f32,
    pub swing: f32,
    pub key: i8,
    /// Length in sixteenth notes.
    pub steps: u16,
    pub drums: Vec<DrumHit>,
    pub bassline: Vec<BassNote>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SongPad {
    pub label: String,
    pub pattern: usize,
    pub job: PadJob,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArrStep {
    pub pattern: usize,
    pub repeats: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Song {
    pub name: String,
    pub bpm: f32,
    pub swing: f32,
    pub key: i8,
    pub pads: Vec<SongPad>,
    pub arrangement: Vec<ArrStep>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct FxSettings {
    pub comp_on: bool,
    pub comp: f32,
    pub tone: f32,
    pub resonance: f32,
    pub res_freq: f32,
    pub lofi: f32,
    pub space_on: bool,
    /// 0 room, 1 hall, 2 plate, 3 spring, 4 delay, 5 chorus, 6 flanger
    pub space: u8,
    pub space_mix: f32,
    pub space_time: f32,
}

impl Default for FxSettings {
    fn default() -> Self {
        Self {
            comp_on: true,
            comp: 0.38,
            tone: 0.08,
            resonance: 0.0,
            res_freq: 800.0,
            lofi: 0.0,
            space_on: true,
            space: 0,
            space_mix: 0.22,
            space_time: 0.45,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Document {
    pub patterns: Vec<Pattern>,
    pub songs: Vec<Song>,
    pub kits: Vec<Kit>,
    pub basses: Vec<BassProgram>,
    pub fx: FxSettings,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "t", rename_all = "snake_case")]
pub enum Command {
    Play,
    Stop,
    ToggleRec,
    SetMode { mode: u8 },
    SetPart { drum: bool },
    SetLiveDrums { on: bool },
    SetPattern { index: usize },
    SetSong { index: usize },
    SetTempo { bpm: f32 },
    SetSwing { amount: f32 },
    SetKey { key: i8 },
    SetKit { index: usize },
    SetBass { index: usize },
    SetBassMute { mute: bool },
    SetMetronome { on: bool },
    SetQuantize { division: u8 },
    SetOctave { octave: i8 },
    PadDown { pad: u8, vel: f32 },
    PadUp { pad: u8 },
    Section { pad: u8 },
    Groove { pad: u8 },
    ToggleStep { pad: u8, step: usize, vel: u8 },
    SetBassStep { step: usize, note: i16, dur: u8 },
    ClearDrums,
    ClearBass,
    DuplicatePattern,
    SetFx { fx: FxSettings },
    Midi { channel: u8, note: u8, vel: f32, on: bool },
    LoadDocument { json: String },
    ResetFactory,
    Panic,
}

#[derive(Clone, Copy, Debug)]
pub struct HostPos {
    pub playing: bool,
    pub tempo: f32,
    pub beat: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct MidiEvt {
    pub channel: u8,
    pub note: u8,
    pub vel: f32,
    pub on: bool,
    pub frame: u32,
}
