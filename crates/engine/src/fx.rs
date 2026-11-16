use crate::doc::FxSettings;

pub struct Fx {
    sr: f32,
    lp_d: f32,
    lp_b: f32,
    env_d: f32,
    env_b: f32,
    tilt: f32,
    svf_lp: f32,
    svf_bp: f32,
    hold_n: u32,
    held_l: f32,
    held_r: f32,
    comb_l: Vec<Vec<f32>>,
    comb_r: Vec<Vec<f32>>,
    comb_il: [usize; 4],
    comb_ir: [usize; 4],
    damp_l: [f32; 4],
    damp_r: [f32; 4],
    ap_l: Vec<Vec<f32>>,
    ap_r: Vec<Vec<f32>>,
    ap_il: [usize; 2],
    ap_ir: [usize; 2],
    delay_l: Vec<f32>,
    delay_r: Vec<f32>,
    delay_i: usize,
    chorus_phase: f32,
    chorus_l: Vec<f32>,
    chorus_r: Vec<f32>,
    chorus_i: usize,
}

impl Fx {
    pub fn new(sr: f32) -> Self {
        let mut fx = Self {
            sr: 48_000.0,
            lp_d: 0.0,
            lp_b: 0.0,
            env_d: 0.0,
            env_b: 0.0,
            tilt: 0.0,
            svf_lp: 0.0,
            svf_bp: 0.0,
            hold_n: 0,
            held_l: 0.0,
            held_r: 0.0,
            comb_l: Vec::new(),
            comb_r: Vec::new(),
            comb_il: [0; 4],
            comb_ir: [0; 4],
            damp_l: [0.0; 4],
            damp_r: [0.0; 4],
            ap_l: Vec::new(),
            ap_r: Vec::new(),
            ap_il: [0; 2],
            ap_ir: [0; 2],
            delay_l: Vec::new(),
            delay_r: Vec::new(),
            delay_i: 0,
            chorus_phase: 0.0,
            chorus_l: Vec::new(),
            chorus_r: Vec::new(),
            chorus_i: 0,
        };
        fx.resize(sr);
        fx
    }

    pub fn resize(&mut self, sr: f32) {
        self.sr = sr.max(8_000.0);
        let sec = |s: f32| ((s * self.sr) as usize).max(8);
        let ls = [0.029, 0.037, 0.041, 0.053];
        let rs = [0.031, 0.039, 0.047, 0.059];
        self.comb_l = ls.into_iter().map(|s| vec![0.0; sec(s)]).collect();
        self.comb_r = rs.into_iter().map(|s| vec![0.0; sec(s)]).collect();
        self.ap_l = vec![vec![0.0; sec(0.005)], vec![0.0; sec(0.0017)]];
        self.ap_r = vec![vec![0.0; sec(0.0056)], vec![0.0; sec(0.0013)]];
        let dlen = sec(2.0);
        self.delay_l = vec![0.0; dlen];
        self.delay_r = vec![0.0; dlen];
        let clen = sec(0.04);
        self.chorus_l = vec![0.0; clen];
        self.chorus_r = vec![0.0; clen];
        self.comb_il = [0; 4];
        self.comb_ir = [0; 4];
        self.ap_il = [0; 2];
        self.ap_ir = [0; 2];
        self.delay_i = 0;
        self.chorus_i = 0;
    }

    pub fn process(
        &mut self,
        l: f32,
        r: f32,
        send_l: f32,
        send_r: f32,
        fx: &FxSettings,
        tempo: f32,
    ) -> (f32, f32) {
        let (l, r) = self.comp_eq(l, r, fx);
        let (l, r) = self.res_lofi(l, r, fx);
        if !fx.space_on || fx.space_mix <= 0.001 {
            return soft(l, r);
        }
        let wet = match fx.space {
            4 => self.delay(send_l, send_r, fx, tempo),
            5 => self.chorus(send_l, send_r, false),
            6 => self.chorus(send_l, send_r, true),
            kind => self.reverb(send_l, send_r, kind, fx.space_time),
        };
        let mix = fx.space_mix.clamp(0.0, 1.0);
        soft(l + wet.0 * mix, r + wet.1 * mix)
    }

    fn comp_eq(&mut self, l: f32, r: f32, fx: &FxSettings) -> (f32, f32) {
        let mono = 0.5 * (l + r);
        let a = 1.0 - (-2.0 * std::f32::consts::PI * 180.0 / self.sr).exp();
        self.lp_d += a * (l - self.lp_d);
        self.lp_b += a * (mono - self.lp_b);
        let low_l = self.lp_d;
        let high_l = l - self.lp_d;
        let amount = if fx.comp_on { fx.comp.clamp(0.0, 1.0) } else { 0.0 };
        let c_low = comp_gain(&mut self.env_d, low_l, amount);
        let c_high = comp_gain(&mut self.env_b, high_l, amount * 0.85);
        let mut out_l = low_l * c_low + high_l * c_high;
        let mut out_r = r * (0.5 * (c_low + c_high));
        let tone = fx.tone.clamp(-1.0, 1.0);
        self.tilt += 0.08 * (out_l - self.tilt);
        let high = out_l - self.tilt;
        out_l += tone * high * 0.9;
        out_r += tone * (out_r - self.tilt) * 0.5;
        if tone < 0.0 {
            out_l = out_l * (1.0 + tone * 0.3) + self.tilt * (-tone) * 0.4;
        }
        (out_l, out_r)
    }

    fn res_lofi(&mut self, mut l: f32, mut r: f32, fx: &FxSettings) -> (f32, f32) {
        if fx.resonance > 0.01 {
            let freq = fx.res_freq.clamp(80.0, 8000.0);
            let f = (2.0 * (std::f32::consts::PI * freq / self.sr).sin()).clamp(0.0, 0.8);
            let q = (1.0 - fx.resonance.clamp(0.0, 1.0) * 0.93).max(0.08);
            let x = 0.5 * (l + r);
            self.svf_lp += f * self.svf_bp;
            let hp = x - self.svf_lp - q * self.svf_bp;
            self.svf_bp += f * hp;
            let band = self.svf_bp * fx.resonance * 1.4;
            l += band;
            r += band;
        }
        if fx.lofi > 0.01 {
            self.hold_n += 1;
            let hold = (1.0 + fx.lofi * 40.0) as u32;
            if self.hold_n >= hold {
                self.hold_n = 0;
                let steps = (4.0 + (1.0 - fx.lofi) * 80.0).max(3.0);
                self.held_l = (l * steps).round() / steps;
                self.held_r = (r * steps).round() / steps;
            }
            let m = fx.lofi.clamp(0.0, 1.0);
            l = l * (1.0 - m) + self.held_l * m;
            r = r * (1.0 - m) + self.held_r * m;
        }
        (l, r)
    }

    fn reverb(&mut self, l: f32, r: f32, kind: u8, time: f32) -> (f32, f32) {
        let feedback = match kind {
            1 => 0.78 + time * 0.16,
            2 => 0.72 + time * 0.14,
            3 => 0.62 + time * 0.12,
            _ => 0.55 + time * 0.2,
        }
        .clamp(0.3, 0.92);
        let damp = match kind {
            3 => 0.15,
            1 => 0.28,
            _ => 0.38,
        };
        let mut acc_l = 0.0;
        let mut acc_r = 0.0;
        for i in 0..4 {
            acc_l += comb(&mut self.comb_l[i], &mut self.comb_il[i], l, feedback, &mut self.damp_l[i], damp);
            acc_r += comb(&mut self.comb_r[i], &mut self.comb_ir[i], r, feedback, &mut self.damp_r[i], damp);
        }
        acc_l *= 0.28;
        acc_r *= 0.28;
        for i in 0..2 {
            acc_l = allpass(&mut self.ap_l[i], &mut self.ap_il[i], acc_l, 0.7);
            acc_r = allpass(&mut self.ap_r[i], &mut self.ap_ir[i], acc_r, 0.7);
        }
        (acc_l, acc_r)
    }

    fn delay(&mut self, l: f32, r: f32, fx: &FxSettings, tempo: f32) -> (f32, f32) {
        let beats = [0.25, 0.5, 0.75, 1.0, 1.5][(fx.space_time * 4.0) as usize % 5];
        let samples = ((60.0 / tempo.max(40.0)) * beats * self.sr) as usize;
        let len = self.delay_l.len();
        let samples = samples.clamp(1, len - 1);
        let read = (self.delay_i + len - samples) % len;
        let yl = self.delay_l[read];
        let yr = self.delay_r[read];
        self.delay_l[self.delay_i] = l + yl * 0.45;
        self.delay_r[self.delay_i] = r + yr * 0.45;
        self.delay_i = (self.delay_i + 1) % len;
        (yl, yr)
    }

    fn chorus(&mut self, l: f32, r: f32, flanger: bool) -> (f32, f32) {
        let len = self.chorus_l.len();
        self.chorus_l[self.chorus_i] = l;
        self.chorus_r[self.chorus_i] = r;
        let rate = if flanger { 0.35 } else { 0.6 };
        self.chorus_phase += rate / self.sr;
        if self.chorus_phase > 1.0 {
            self.chorus_phase -= 1.0;
        }
        let modu = (self.chorus_phase * std::f32::consts::TAU).sin();
        let depth = if flanger { 0.003 } else { 0.012 };
        let base = if flanger { 0.002 } else { 0.014 };
        let delay = ((base + depth * modu) * self.sr).max(1.0);
        let read_f = self.chorus_i as f32 - delay;
        let i0 = read_f.floor();
        let frac = read_f - i0;
        let a = ((i0 as isize).rem_euclid(len as isize)) as usize;
        let b = (a + 1) % len;
        let fb = if flanger { 0.55 } else { 0.15 };
        let yl = self.chorus_l[a] * (1.0 - frac) + self.chorus_l[b] * frac;
        let yr = self.chorus_r[a] * (1.0 - frac) + self.chorus_r[b] * frac;
        self.chorus_l[self.chorus_i] += yl * fb;
        self.chorus_r[self.chorus_i] += yr * fb;
        self.chorus_i = (self.chorus_i + 1) % len;
        (yl, yr)
    }
}

fn comp_gain(env: &mut f32, x: f32, amount: f32) -> f32 {
    if amount <= 0.001 {
        return 1.0;
    }
    let a = x.abs();
    let coef = if a > *env { 0.04 } else { 0.002 };
    *env += coef * (a - *env);
    let thresh = 0.32 - amount * 0.18;
    let over = (*env / thresh.max(0.05)).max(1.0);
    over.powf(-0.65 * amount) * (1.0 + amount * 0.55)
}

fn comb(buf: &mut [f32], idx: &mut usize, x: f32, feedback: f32, damp: &mut f32, coef: f32) -> f32 {
    let y = buf[*idx];
    *damp = y * (1.0 - coef) + *damp * coef;
    buf[*idx] = x + *damp * feedback;
    *idx = (*idx + 1) % buf.len();
    y
}

fn allpass(buf: &mut [f32], idx: &mut usize, x: f32, g: f32) -> f32 {
    let delayed = buf[*idx];
    let out = -g * x + delayed;
    buf[*idx] = x + g * delayed;
    *idx = (*idx + 1) % buf.len();
    out
}

fn soft(l: f32, r: f32) -> (f32, f32) {
    ((l * 1.15).tanh() * 0.92, (r * 1.15).tanh() * 0.92)
}
