use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, EguiState};
use parking_lot::Mutex;
use section_engine::{Command, Engine, HostPos, View, PADS};
use std::sync::{mpsc, Arc};

struct Section {
    params: Arc<SectionParams>,
    editor: Arc<EguiState>,
    engine: Engine,
    view: Arc<Mutex<View>>,
    cmd_tx: mpsc::Sender<Command>,
    cmd_rx: mpsc::Receiver<Command>,
    sr: f32,
    seen_swing: f32,
    seen_song: i32,
    seen_pattern: i32,
    doc_hash: u64,
    blocks: u32,
}

#[derive(Params)]
struct SectionParams {
    #[id = "output"]
    pub output: FloatParam,
    #[id = "swing"]
    pub swing: FloatParam,
    #[id = "song"]
    pub song: IntParam,
    #[id = "pattern"]
    pub pattern: IntParam,
    #[persist = "document"]
    pub document: Mutex<String>,
}

impl Default for SectionParams {
    fn default() -> Self {
        Self {
            output: FloatParam::new(
                "Output",
                util::db_to_gain(-2.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-36.0),
                    max: util::db_to_gain(6.0),
                    factor: FloatRange::gain_skew_factor(-36.0, 6.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(40.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(1))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            swing: FloatParam::new("Swing", 0.0, FloatRange::Linear { min: 0.0, max: 0.7 })
                .with_unit("%")
                .with_value_to_string(formatters::v2s_f32_percentage(0))
                .with_string_to_value(formatters::s2v_f32_percentage()),
            song: IntParam::new("Song", 0, IntRange::Linear { min: 0, max: 31 }),
            pattern: IntParam::new("Pattern", 0, IntRange::Linear { min: 0, max: 127 }),
            document: Mutex::new(String::new()),
        }
    }
}

impl Default for Section {
    fn default() -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let params = SectionParams::default();
        Self {
            seen_swing: params.swing.value(),
            seen_song: params.song.value(),
            seen_pattern: params.pattern.value(),
            params: Arc::new(params),
            editor: EguiState::from_size(980, 680),
            engine: Engine::new(44_100.0),
            view: Arc::new(Mutex::new(View::new())),
            cmd_tx,
            cmd_rx,
            sr: 44_100.0,
            doc_hash: 0,
            blocks: 0,
        }
    }
}

impl Section {
    fn apply_params(&mut self) {
        let swing = self.params.swing.value();
        if (swing - self.seen_swing).abs() > 0.0008 {
            self.seen_swing = swing;
            self.engine.command(Command::SetSwing { amount: swing });
        }
        let song = self.params.song.value();
        if song != self.seen_song {
            self.seen_song = song;
            self.engine.command(Command::SetSong { index: song as usize });
        }
        let pattern = self.params.pattern.value();
        if pattern != self.seen_pattern {
            self.seen_pattern = pattern;
            self.engine.command(Command::SetPattern { index: pattern as usize });
        }
    }

    fn persist_if_needed(&mut self) {
        if self.engine.take_dirty() {
            let json = self.engine.export_json();
            self.doc_hash = hash_str(&json);
            *self.params.document.lock() = json;
        }
        self.blocks = self.blocks.wrapping_add(1);
        if self.blocks % 64 == 0 {
            let guard = self.params.document.lock();
            if !guard.is_empty() {
                let hash = hash_str(&guard);
                if hash != self.doc_hash {
                    let json = guard.clone();
                    drop(guard);
                    self.engine.load_json(&json);
                    self.doc_hash = hash;
                }
            }
        }
    }
}

impl Plugin for Section {
    const NAME: &'static str = "Section";
    const VENDOR: &'static str = "Pedro Muller";
    const URL: &'static str = "https://pedro-muller.com/drum-machine";
    const EMAIL: &'static str = "work@pedro-muller.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: None,
        main_output_channels: NonZeroU32::new(2),
        aux_input_ports: &[],
        aux_output_ports: &[],
        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.sr = buffer_config.sample_rate;
        self.engine.set_sample_rate(buffer_config.sample_rate);
        let guard = self.params.document.lock();
        if !guard.is_empty() && self.engine.load_json(&guard) {
            self.doc_hash = hash_str(&guard);
        }
        true
    }

    fn reset(&mut self) {
        self.engine.command(Command::Panic);
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let tx = self.cmd_tx.clone();
        let view = self.view.clone();
        let params = self.params.clone();
        create_egui_editor(
            self.editor.clone(),
            (),
            |_, _| {},
            move |ctx, setter, _state| {
                ctx.request_repaint();
                let snapshot = view.lock().clone();
                egui::CentralPanel::default().show(ctx, |ui| {
                    draw_ui(ui, setter, &params, &tx, &snapshot);
                });
            },
        )
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        while let Ok(cmd) = self.cmd_rx.try_recv() {
            self.engine.command(cmd);
        }
        self.apply_params();

        let transport = context.transport();
        let tempo = transport.tempo.unwrap_or(120.0) as f32;
        let playing = transport.playing;
        let start_beat = transport.pos_beats().unwrap_or(0.0);
        let bps = tempo as f64 / 60.0;
        let sr = self.sr as f64;

        let mut next_event = context.next_event();
        for (sample_id, channels) in buffer.iter_samples().enumerate() {
            let timing = sample_id as u32;
            while let Some(event) = next_event {
                if event.timing() > timing {
                    break;
                }
                match event {
                    NoteEvent::NoteOn { channel, note, velocity, .. } => {
                        self.engine.midi(channel, note, velocity, true);
                    }
                    NoteEvent::NoteOff { channel, note, .. } => {
                        self.engine.midi(channel, note, 0.0, false);
                    }
                    _ => {}
                }
                next_event = context.next_event();
            }

            let host = HostPos {
                playing,
                tempo,
                beat: start_beat + sample_id as f64 * bps / sr,
            };
            let (mut l, mut r) = self.engine.process_sample(Some(host));
            let gain = self.params.output.smoothed.next();
            l *= gain;
            r *= gain;
            let mut slots = channels.into_iter();
            if let Some(left) = slots.next() {
                *left = l;
            }
            if let Some(right) = slots.next() {
                *right = r;
            }
        }

        if self.blocks % 8 == 0 {
            let mut view = self.view.lock();
            self.engine.write_view(&mut view);
        }
        self.persist_if_needed();
        ProcessStatus::Normal
    }
}

fn draw_ui(
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    params: &SectionParams,
    tx: &mpsc::Sender<Command>,
    view: &View,
) {
    ui.horizontal(|ui| {
        ui.heading("Section");
        ui.label(name(&view.names[1]));
        ui.label(format!("{:5.1} bpm", view.tempo));
        ui.label(format!("bar {}", view.bar));
        if view.playing {
            ui.colored_label(egui::Color32::from_rgb(230, 170, 70), "PLAY");
        }
    });
    ui.horizontal(|ui| {
        if ui.button(if view.playing { "Stop" } else { "Play" }).clicked() {
            let _ = tx.send(if view.playing { Command::Stop } else { Command::Play });
        }
        if ui.button(if view.recording { "Recording" } else { "Record" }).clicked() {
            let _ = tx.send(Command::ToggleRec);
        }
        if ui.button("Pattern").clicked() {
            let _ = tx.send(Command::SetMode { mode: 0 });
        }
        if ui.button("Song").clicked() {
            let _ = tx.send(Command::SetMode { mode: 1 });
        }
        if ui.button("Groove").clicked() {
            let _ = tx.send(Command::SetMode { mode: 2 });
        }
        if ui.button(if view.drum_part { "Drums" } else { "Bass" }).clicked() {
            let _ = tx.send(Command::SetPart { drum: !view.drum_part });
        }
        if ui.button(if view.bass_mute { "Bass muted" } else { "Mute bass" }).clicked() {
            let _ = tx.send(Command::SetBassMute { mute: !view.bass_mute });
        }
        ui.label(name(&view.names[2]));
        ui.label(name(&view.names[3]));
    });

    ui.separator();
    ui.label("Song sections — hit these while Ableton is playing, or press Play here to preview.");
    ui.horizontal(|ui| {
        for pad in 0..6 {
            let label = name(&view.song_pads[pad]);
            let text = if label.is_empty() { "—".to_string() } else { label };
            if ui.add_sized([120.0, 64.0], egui::Button::new(text)).clicked() {
                let _ = tx.send(Command::Section { pad: pad as u8 });
            }
        }
    });

    ui.add_space(8.0);
    ui.label(if view.drum_part {
        "Drum pads. MIDI channel 10 plays drums. Any other channel plays bass."
    } else {
        "Bass pads, low to high. Hold for note length when recording."
    });
    let labels = [
        "KICK", "SNARE", "HAT", "OPEN", "RIM", "BELL", "TAMB", "HT", "MT", "LT", "RIDE", "CRASH", "CLAP",
    ];
    ui.horizontal(|ui| {
        for pad in 0..7 {
            pad_button(ui, tx, view, pad, labels[pad]);
        }
    });
    ui.horizontal(|ui| {
        for pad in 7..PADS {
            pad_button(ui, tx, view, pad, labels[pad]);
        }
    });

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        if ui.button("Kit −").clicked() {
            let _ = tx.send(Command::SetKit { index: view.kit.saturating_sub(1) as usize });
        }
        if ui.button("Kit +").clicked() {
            let _ = tx.send(Command::SetKit { index: view.kit as usize + 1 });
        }
        if ui.button("Bass −").clicked() {
            let _ = tx.send(Command::SetBass { index: view.bass.saturating_sub(1) as usize });
        }
        if ui.button("Bass +").clicked() {
            let _ = tx.send(Command::SetBass { index: view.bass as usize + 1 });
        }
        if ui.button("Key −").clicked() {
            let _ = tx.send(Command::SetKey { key: (view.key + 11) % 12 });
        }
        if ui.button("Key +").clicked() {
            let _ = tx.send(Command::SetKey { key: (view.key + 1) % 12 });
        }
        if ui.button("Song −").clicked() {
            let next = view.song.saturating_sub(1);
            setter.begin_set_parameter(&params.song);
            setter.set_parameter(&params.song, i32::from(next));
            setter.end_set_parameter(&params.song);
            let _ = tx.send(Command::SetSong { index: next as usize });
        }
        if ui.button("Song +").clicked() {
            let next = view.song + 1;
            setter.begin_set_parameter(&params.song);
            setter.set_parameter(&params.song, i32::from(next));
            setter.end_set_parameter(&params.song);
            let _ = tx.send(Command::SetSong { index: next as usize });
        }
    });

    ui.horizontal(|ui| {
        let mut swing = view.swing;
        if ui.add(egui::Slider::new(&mut swing, 0.0..=0.7).text("Swing")).changed() {
            setter.begin_set_parameter(&params.swing);
            setter.set_parameter(&params.swing, swing);
            setter.end_set_parameter(&params.swing);
            let _ = tx.send(Command::SetSwing { amount: swing });
        }
    });
    ui.label("Follows Ableton tempo and play. Channel 10 is the drum kit. Preview with Play when the transport is stopped.");
    let _ = setter;
}

fn pad_button(ui: &mut egui::Ui, tx: &mpsc::Sender<Command>, view: &View, pad: usize, label: &str) {
    let response = ui.add_sized([78.0, 56.0], egui::Button::new(label));
    if response.drag_started() || response.clicked() {
        let _ = tx.send(if view.mode == 2 {
            Command::Groove { pad: pad as u8 }
        } else {
            Command::PadDown { pad: pad as u8, vel: 0.9 }
        });
    }
    if response.drag_stopped() {
        let _ = tx.send(Command::PadUp { pad: pad as u8 });
    }
}

fn name(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|b| *b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).trim().to_string()
}

fn hash_str(value: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in value.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

impl ClapPlugin for Section {
    const CLAP_ID: &'static str = "com.pedromuller.section";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Drum and bass rhythm section");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::Instrument, ClapFeature::DrumMachine];
}

impl Vst3Plugin for Section {
    const VST3_CLASS_ID: [u8; 16] = *b"PmSectionDrum001";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Instrument, Vst3SubCategory::Drum];
}

nih_export_vst3!(Section);
