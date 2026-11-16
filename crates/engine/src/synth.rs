use crate::doc::{BassProgram, PadAssign};

pub const KICK: u8 = 0;
pub const SNARE: u8 = 1;
pub const CLAP: u8 = 2;
pub const HAT: u8 = 3;
pub const OPEN: u8 = 4;
pub const RIM: u8 = 5;
pub const COWBELL: u8 = 6;
pub const TAMB: u8 = 7;
pub const TOM_H: u8 = 8;
pub const TOM_M: u8 = 9;
pub const TOM_L: u8 = 10;
pub const RIDE: u8 = 11;
pub const CRASH: u8 = 12;
pub const SHAKER: u8 = 13;
pub const CONGA: u8 = 14;
pub const CLAVE: u8 = 15;
pub const SNAP: u8 = 16;
pub const KICK808: u8 = 17;
pub const SNARE808: u8 = 18;
pub const HAT808: u8 = 19;
pub const TOM808: u8 = 20;
pub const STICK: u8 = 21;
pub const BRUSH: u8 = 22;
pub const RIDE_BELL: u8 = 23;

const TAU: f32 = std::f32::consts::TAU;

#[derive(Clone)]
#[allow(dead_code)]
pub struct Voice {
    pub on: bool,
    pub inst: u8,
    pub vel: f32,
    pub tune: f32,
    pub decay: f32,
    pub pan: f32,
    pub send: f32,
    pub level: f32,
    pub t: f32,
    pub phase: f32,
    pub phase2: f32,
    pub phase3: f32,
    pub lp: f32,
    pub hp: f32,
    pub nstate: u32,
    pub env: f32,
}

impl Voice {
    pub fn silent() -> Self {
        Self {
            on: false,
            inst: 0,
            vel: 0.0,
            tune: 1.0,
            decay: 1.0,
            pan: 0.0,
            send: 0.0,
            level: 1.0,
            t: 0.0,
            phase: 0.0,
            phase2: 0.0,
            phase3: 0.0,
            lp: 0.0,
            hp: 0.0,
            nstate: 1,
            env: 0.0,
        }
    }
}

pub fn trigger(voice: &mut Voice, inst: u8, vel: f32, pad: &PadAssign, seed: u32) {
    *voice = Voice::silent();
    voice.on = true;
    voice.inst = inst;
    voice.vel = vel.clamp(0.05, 1.0);
    voice.tune = 2.0_f32.powf(pad.tune / 12.0);
    voice.decay = pad.decay.max(0.15);
    voice.pan = pad.pan.clamp(-1.0, 1.0);
    voice.send = pad.send.clamp(0.0, 1.0);
    voice.level = pad.level.clamp(0.0, 1.5);
    voice.nstate = seed | 1;
}

fn noise(state: &mut u32) -> f32 {
    *state = state.wrapping_mul(1664525).wrapping_add(1013904223);
    (*state >> 9) as f32 / 8_388_608.0 - 1.0
}

fn sin_phase(phase: f32) -> f32 {
    (phase * TAU).sin()
}

pub fn render_voice(v: &mut Voice, sr: f32) -> f32 {
    if !v.on {
        return 0.0;
    }
    let t = v.t;
    let dt = 1.0 / sr;
    let tune = v.tune;
    let sample = match v.inst {
        KICK => {
            let f = 52.0 * tune * (1.0 + 16.0 * (-t * 26.0).exp());
            v.phase += f * dt;
            let body = sin_phase(v.phase);
            let click = noise(&mut v.nstate) * (-t * 280.0).exp();
            let amp = (-t * (5.2 / v.decay)).exp();
            (body * 1.05 + click * 0.42) * amp
        }
        KICK808 => {
            let f = 46.0 * tune * (1.0 + 11.0 * (-t * 16.0).exp());
            v.phase += f * dt;
            let amp = (-t * (1.35 / v.decay)).exp();
            sin_phase(v.phase) * amp * 1.15
        }
        SNARE => {
            v.phase += 188.0 * tune * dt;
            let tone = sin_phase(v.phase) * (-t * 32.0).exp();
            let n = noise(&mut v.nstate);
            v.lp += 0.42 * (n - v.lp);
            let hp = n - v.lp;
            let namp = (-t * (11.0 / v.decay)).exp();
            tone * 0.55 + hp * namp * 0.95
        }
        SNARE808 => {
            v.phase += 176.0 * tune * dt;
            let tone = sin_phase(v.phase) * (-t * 18.0).exp();
            let n = noise(&mut v.nstate);
            v.lp += 0.35 * (n - v.lp);
            (tone * 0.85 + (n - v.lp) * (-t * (8.0 / v.decay)).exp()) * 0.9
        }
        CLAP | SNAP => {
            let n = noise(&mut v.nstate);
            v.lp += 0.55 * (n - v.lp);
            let bp = n - v.lp;
            let burst = if t < 0.004 || (t > 0.009 && t < 0.014) || (t > 0.020 && t < 0.028) {
                1.0
            } else {
                0.25
            };
            let tail = if t < 0.028 {
                1.0
            } else {
                (-(t - 0.028) * (16.0 / v.decay)).exp()
            };
            let mul = if v.inst == SNAP { 0.7 } else { 1.0 };
            bp * burst * tail * mul
        }
        HAT | OPEN | HAT808 | SHAKER | BRUSH => {
            let closed = matches!(v.inst, HAT | HAT808);
            let metal_f = if v.inst == BRUSH { 2200.0 } else { 6400.0 };
            v.phase += metal_f * tune * dt;
            v.phase2 += metal_f * 1.47 * tune * dt;
            let metal = sin_phase(v.phase).signum() * 0.25 + sin_phase(v.phase2).signum() * 0.18;
            let n = noise(&mut v.nstate);
            let coef = if v.inst == SHAKER { 0.35 } else { 0.72 };
            v.lp += coef * (n - v.lp);
            let hp = n - v.lp;
            let d = if closed {
                62.0 / v.decay
            } else if v.inst == OPEN {
                7.5 / v.decay
            } else if v.inst == BRUSH {
                9.0 / v.decay
            } else {
                18.0 / v.decay
            };
            (metal + hp * 0.85) * (-t * d).exp()
        }
        RIM | STICK | CLAVE => {
            let f = match v.inst {
                CLAVE => 2150.0,
                STICK => 1400.0,
                _ => 620.0,
            };
            v.phase += f * tune * dt;
            let n = noise(&mut v.nstate) * (-t * 220.0).exp();
            (sin_phase(v.phase) * 0.7 + n * 0.35) * (-t * (48.0 / v.decay)).exp()
        }
        COWBELL => {
            v.phase += 587.0 * tune * dt;
            v.phase2 += 845.0 * tune * dt;
            let s = sin_phase(v.phase).signum() * 0.35 + sin_phase(v.phase2).signum() * 0.28;
            s * (-t * (9.0 / v.decay)).exp()
        }
        TAMB => {
            let n = noise(&mut v.nstate);
            v.lp += 0.5 * (n - v.lp);
            let mut amp = (-t * (14.0 / v.decay)).exp();
            if t > 0.045 && t < 0.07 {
                amp += 0.45 * (-(t - 0.045) * 30.0).exp();
            }
            (n - v.lp) * amp
        }
        TOM_H | TOM_M | TOM_L | TOM808 | CONGA => {
            let base = match v.inst {
                TOM_H => 196.0,
                TOM_M => 140.0,
                TOM_L => 98.0,
                TOM808 => 110.0,
                _ => 168.0,
            };
            let drop = if v.inst == CONGA { 1.6 } else { 2.8 };
            let f = base * tune * (1.0 + drop * (-t * 22.0).exp());
            v.phase += f * dt;
            let n = noise(&mut v.nstate) * (-t * 90.0).exp();
            let dec = if v.inst == TOM808 { 2.2 } else { 4.4 };
            (sin_phase(v.phase) * 0.95 + n * 0.18) * (-t * (dec / v.decay)).exp()
        }
        RIDE | RIDE_BELL | CRASH => {
            let long = v.inst == CRASH;
            v.phase += 320.0 * tune * dt;
            v.phase2 += 740.0 * tune * dt;
            v.phase3 += 1280.0 * tune * dt;
            let metal = sin_phase(v.phase) * 0.22
                + sin_phase(v.phase2) * 0.16
                + sin_phase(v.phase3).signum() * 0.12;
            let n = noise(&mut v.nstate);
            v.lp += 0.25 * (n - v.lp);
            let d = if long {
                1.15 / v.decay
            } else if v.inst == RIDE_BELL {
                6.5 / v.decay
            } else {
                2.4 / v.decay
            };
            (metal + (n - v.lp) * 0.55) * (-t * d).exp()
        }
        _ => 0.0,
    };
    v.t += dt;
    let out = sample * v.vel * v.level;
    if v.t > 4.5 || (v.t > 0.05 && sample.abs() < 0.00005 && v.inst != CRASH) {
        if v.t > 0.2 && sample.abs() < 0.0004 {
            v.on = false;
        }
    }
    if !out.is_finite() {
        v.on = false;
        return 0.0;
    }
    out
}

#[derive(Clone)]
pub struct BassVoice {
    pub gate: bool,
    pub active: bool,
    pub note: u8,
    pub freq: f32,
    pub target: f32,
    pub phase: f32,
    pub phase2: f32,
    pub env: f32,
    pub fenv: f32,
    pub lp: f32,
    pub bp: f32,
    pub vel: f32,
    pub program: usize,
    pub nstate: u32,
    pub age: f32,
}

impl BassVoice {
    pub fn new() -> Self {
        Self {
            gate: false,
            active: false,
            note: 36,
            freq: 60.0,
            target: 60.0,
            phase: 0.0,
            phase2: 0.13,
            env: 0.0,
            fenv: 0.0,
            lp: 0.0,
            bp: 0.0,
            vel: 0.8,
            program: 0,
            nstate: 3,
            age: 0.0,
        }
    }

    pub fn note_on(&mut self, note: u8, vel: f32, program: usize, seed: u32) {
        self.note = note;
        self.vel = vel.clamp(0.05, 1.0);
        self.program = program;
        self.target = 440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0);
        if !self.active {
            self.freq = self.target;
        }
        self.gate = true;
        self.active = true;
        self.fenv = 1.0;
        self.age = 0.0;
        self.nstate = seed | 1;
        self.env = self.env.max(0.2);
    }

    pub fn note_off(&mut self, note: u8) {
        if self.note == note {
            self.gate = false;
        }
    }

    pub fn release(&mut self) {
        self.gate = false;
    }

    pub fn render(&mut self, prog: &BassProgram, sr: f32) -> f32 {
        if !self.active {
            return 0.0;
        }
        let dt = 1.0 / sr;
        let glide = (0.0004 + prog.glide * 0.02).clamp(0.0004, 0.08);
        self.freq += (self.target - self.freq) * glide;
        let freq = self.freq.max(20.0);
        self.phase += freq * dt;
        self.phase2 += freq * 1.007 * dt;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        if self.phase2 >= 1.0 {
            self.phase2 -= 1.0;
        }
        let p = self.phase;
        let saw = 2.0 * p - 1.0;
        let sqr = if p < 0.5 { 1.0 } else { -1.0 };
        let sine = sin_phase(p);
        let tri = {
            let x = (p * 2.0 - 1.0).abs();
            1.0 - 2.0 * x
        };
        let n = noise(&mut self.nstate);
        let osc = match prog.osc {
            1 => saw * 0.8,
            2 => saw * 0.55 + sqr * 0.25 + n * (-self.age * 40.0).exp() * prog.punch,
            3 => saw * 0.7,
            4 => sine * 0.75 + tri * 0.35,
            5 => sine,
            6 => saw * 0.6 + (2.0 * self.phase2 - 1.0) * 0.35,
            7 => saw * 0.45 + (2.0 * self.phase2 - 1.0) * 0.45,
            8 => sqr * 0.55,
            9 => sine * 0.8 + tri * 0.15,
            10 => sine + sin_phase(p * 2.0) * 0.35 + sin_phase(p * 3.0) * 0.15,
            _ => sine * 0.72 + tri * 0.38 + n * (-self.age * 80.0).exp() * 0.08,
        };
        let sub = sin_phase(p * 0.5) * prog.sub;
        let attack = if self.gate { 0.02 } else { 0.0 };
        if self.gate {
            self.env += (1.0 - self.env) * 0.02;
            if self.env > prog.sustain {
                self.env += (prog.sustain - self.env) * (0.002 + (1.0 - prog.sustain) * 0.01);
            }
        } else {
            self.env *= 1.0 - prog.release.clamp(0.001, 0.2);
        }
        self.fenv *= 0.9992;
        if self.age < 0.03 {
            self.fenv = 1.0;
        }
        let cutoff = (90.0 + prog.cutoff * (0.35 + 0.75 * self.fenv) * (0.65 + 0.5 * self.vel))
            .clamp(40.0, sr * 0.45);
        let f = (2.0 * (std::f32::consts::PI * cutoff / sr).sin()).clamp(0.0, 0.95);
        let q = (1.0 - prog.res.clamp(0.0, 0.95) * 0.9).clamp(0.12, 1.0);
        let input = osc + sub;
        self.lp += f * self.bp;
        let hp = input - self.lp - q * self.bp;
        self.bp += f * hp;
        if self.lp.abs() < 1e-15 {
            self.lp = 0.0;
        }
        if self.bp.abs() < 1e-15 {
            self.bp = 0.0;
        }
        self.age += dt;
        let punch = 1.0 + prog.punch * (-self.age * 30.0).exp();
        let out = (self.lp * 0.85 + input * 0.08) * self.env * self.vel * punch * (1.0 - attack);
        if !self.gate && self.env < 0.001 {
            self.active = false;
        }
        if !out.is_finite() {
            self.active = false;
            self.lp = 0.0;
            self.bp = 0.0;
            return 0.0;
        }
        out.clamp(-1.2, 1.2)
    }
}

pub fn gm_pad(note: u8) -> Option<u8> {
    Some(match note {
        35 | 36 => 0,
        38 | 40 => 1,
        42 | 44 => 2,
        46 => 3,
        37 => 4,
        56 => 5,
        54 => 6,
        48 | 50 => 7,
        45 | 47 => 8,
        41 | 43 => 9,
        51 | 53 | 59 => 10,
        49 | 52 | 55 | 57 => 11,
        39 => 12,
        _ => return None,
    })
}
