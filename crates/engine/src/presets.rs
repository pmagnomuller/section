use crate::doc::*;
use crate::synth::*;

pub fn factory() -> Document {
    let kits = kits();
    let basses = basses();
    let mut patterns = Vec::new();
    let mut songs = Vec::new();
    for style in styles() {
        push_style(&mut patterns, &mut songs, style);
    }
    Document {
        patterns,
        songs,
        kits,
        basses,
        fx: FxSettings::default(),
    }
}

struct Style {
    name: &'static str,
    kit: usize,
    bass: usize,
    bpm: f32,
    swing: f32,
    key: i8,
    two_bar: bool,
    kick: &'static str,
    snare: &'static str,
    hat: &'static str,
    open: &'static str,
    extra: &'static [(u8, &'static str)],
    bassline: &'static str,
}

fn styles() -> Vec<Style> {
    vec![
        Style {
            name: "Funk",
            kit: 2,
            bass: 2,
            bpm: 104.0,
            swing: 0.08,
            key: 9,
            two_bar: false,
            kick: "X..x..X.x..x..x.",
            snare: "....X..o....X...",
            hat: "xxxxxxxxxxxxxxxx",
            open: "......x.....x...",
            extra: &[(4, "..o...o...o...o.")],
            bassline: "a2 . . c3 . . a2 . g2 . . a2 . e2 . .",
        },
        Style {
            name: "Rock",
            kit: 0,
            bass: 1,
            bpm: 118.0,
            swing: 0.0,
            key: 4,
            two_bar: false,
            kick: "X...X..x..X.X...",
            snare: "....X.......X...",
            hat: "x.x.x.x.x.x.x.x.",
            open: "........X.......",
            extra: &[],
            bassline: "e2 . . e2 . . b1 . . . e2 . . . . .",
        },
        Style {
            name: "Hard Rock",
            kit: 1,
            bass: 1,
            bpm: 96.0,
            swing: 0.0,
            key: 2,
            two_bar: false,
            kick: "X...X...X...X.X.",
            snare: "....X.......X...",
            hat: "xxxx.xxxXxxx.xxx",
            open: "........X.......",
            extra: &[],
            bassline: "d2 ~ d2 ~ f2 ~ d2 ~ c2 ~ d2 ~ a1 ~ d2 ~",
        },
        Style {
            name: "Hip-hop",
            kit: 3,
            bass: 5,
            bpm: 92.0,
            swing: 0.12,
            key: 0,
            two_bar: true,
            kick: "X.....x...X..... ....x....X..x...",
            snare: "....X.......X... ....X......oX...",
            hat: "..x...x...x...x. ..x...x...x...x.",
            open: "..............x. ..............x.",
            extra: &[],
            bassline: "c2 ~ ~ ~ . . eb2 ~ . . c2 ~ ~ bb1 ~ . c2 ~ ~ . . . g1 ~ ~ ~ c2 ~ ~ ~ . .",
        },
        Style {
            name: "House",
            kit: 5,
            bass: 6,
            bpm: 124.0,
            swing: 0.0,
            key: 6,
            two_bar: false,
            kick: "X...X...X...X...",
            snare: "....X.......X...",
            hat: "x.x.x.x.x.x.x.x.",
            open: "..x...x...x...x.",
            extra: &[(12, "....X.......X...")],
            bassline: "f#2 . f#3 . f#2 . . . f#2 . c#3 . f#2 . . .",
        },
        Style {
            name: "Disco",
            kit: 7,
            bass: 0,
            bpm: 118.0,
            swing: 0.0,
            key: 0,
            two_bar: false,
            kick: "X...X...X...X...",
            snare: "....X.......X...",
            hat: "xxxxxxxxxxxxxxxx",
            open: "..X...X...X...X.",
            extra: &[],
            bassline: "c2 c3 c2 c3 c2 c3 eb2 c3 c2 c3 bb1 c3 c2 c3 g1 c3",
        },
        Style {
            name: "RnB",
            kit: 9,
            bass: 0,
            bpm: 90.0,
            swing: 0.16,
            key: 2,
            two_bar: true,
            kick: "X......x..X..... ........x...X...",
            snare: "....X.......x.X. ....X.....o.X...",
            hat: "..x...x...x...x. ..x...x...x...x.",
            open: "......x......... ........x.......",
            extra: &[(4, "..............o. ................")],
            bassline: "d2 ~ ~ f2 ~ . . d2 ~ ~ c2 ~ a1 ~ ~ d2 ~ ~ ~ f2 . . . e2 ~ ~ ~ d2 ~ . . .",
        },
        Style {
            name: "Pop",
            kit: 10,
            bass: 0,
            bpm: 120.0,
            swing: 0.0,
            key: 7,
            two_bar: false,
            kick: "X...X..x..X.X...",
            snare: "....X.......X...",
            hat: "xxxxxxxxxxxxxxxx",
            open: "........x.......",
            extra: &[],
            bassline: "g2 . . d2 . g2 . . . b2 . . d2 . . .",
        },
        Style {
            name: "Jazz",
            kit: 6,
            bass: 4,
            bpm: 132.0,
            swing: 0.62,
            key: 5,
            two_bar: true,
            kick: "X...........x... ............x...",
            snare: "....o.......X... ....o.......X...",
            hat: "................ ................",
            open: "................ ................",
            extra: &[(10, "x.xx.xx.x.xx.xx. x.xx.xx.x.xx.x.x")],
            bassline: "f2 ~ ~ ~ a2 ~ ~ ~ c3 ~ ~ ~ a2 ~ ~ ~ g2 ~ ~ ~ bb2 ~ ~ ~ c3 ~ ~ ~ a2 ~ ~ ~",
        },
        Style {
            name: "Reggae",
            kit: 8,
            bass: 9,
            bpm: 78.0,
            swing: 0.0,
            key: 9,
            two_bar: true,
            kick: "........X....... ........X.......",
            snare: "....x...X...x... ....x...X...x...",
            hat: "..x...x...x...x. ..x...x...x...x.",
            open: "......x......... ......x.........",
            extra: &[],
            bassline: "a1 ~ ~ . . . c2 ~ ~ ~ a1 ~ ~ ~ . . a1 ~ ~ . . . e2 ~ ~ ~ d2 ~ ~ ~ . .",
        },
        Style {
            name: "DnB",
            kit: 11,
            bass: 7,
            bpm: 174.0,
            swing: 0.0,
            key: 5,
            two_bar: false,
            kick: "X..........x.x..",
            snare: "....X..X.X..X...",
            hat: "xxxxxxxxxxxxxxxx",
            open: "..............x.",
            extra: &[],
            bassline: "f1 ~ ~ ~ ~ ~ ~ ~ ab1 ~ ~ ~ ~ ~ ~ ~",
        },
        Style {
            name: "Afro",
            kit: 12,
            bass: 0,
            bpm: 116.0,
            swing: 0.06,
            key: 7,
            two_bar: false,
            kick: "x..x..x..x.x..x.",
            snare: "....X.......X...",
            hat: "x.x.x.x.x.x.x.x.",
            open: "......x...x.....",
            extra: &[(8, "x..x...x..x...x."), (5, "..x...x...x...x.")],
            bassline: "g2 . . d2 . . g2 . c2 . . . g2 . bb2 .",
        },
        Style {
            name: "Blues",
            kit: 0,
            bass: 0,
            bpm: 88.0,
            swing: 0.66,
            key: 9,
            two_bar: false,
            kick: "X.......X.......",
            snare: "....X.......X...",
            hat: "x.x.x.x.x.x.x.x.",
            open: "........x.......",
            extra: &[],
            bassline: "a2 ~ ~ ~ c3 ~ ~ ~ e3 ~ ~ ~ a2 ~ ~ ~",
        },
        Style {
            name: "Ballad",
            kit: 13,
            bass: 4,
            bpm: 72.0,
            swing: 0.0,
            key: 0,
            two_bar: true,
            kick: "X...........x... ................",
            snare: "....X.......X... ....X.......X...",
            hat: "................ ................",
            open: "................ ................",
            extra: &[(10, "x.x.x.x.x.x.x.x. x.x.x.x.x.x.x.x.")],
            bassline: "c2 ~ ~ ~ ~ ~ g1 ~ ~ ~ e2 ~ ~ ~ ~ ~ c2 ~ ~ ~ d2 ~ ~ ~ e2 ~ ~ ~ c2 ~ ~ ~",
        },
        // New styles use the rock / blues kits appended below. Existing kits stay as they were.
        Style {
            name: "Indie Rock",
            kit: 16,
            bass: 1,
            bpm: 112.0,
            swing: 0.04,
            key: 2,
            two_bar: false,
            kick: "X.....x.X...X...",
            snare: "....X.......X...",
            hat: "x.x.x.x.x.x.x.x.",
            open: "........X.......",
            extra: &[(4, "..............o.")],
            bassline: "d2 . . a1 . d2 . . . f2 . . a1 . . .",
        },
        Style {
            name: "Alt Rock",
            kit: 17,
            bass: 1,
            bpm: 128.0,
            swing: 0.0,
            key: 7,
            two_bar: false,
            kick: "X...X...X...X.X.",
            snare: "....X.......X...",
            hat: "xxxxxxxxxxxxxxxx",
            open: "..............X.",
            extra: &[(11, "X...............")],
            bassline: "g1 ~ . g1 ~ . bb1 ~ . . g1 ~ . d1 ~ .",
        },
        Style {
            name: "Blues Shuffle",
            kit: 18,
            bass: 0,
            bpm: 96.0,
            swing: 0.68,
            key: 4,
            two_bar: false,
            kick: "X.....x.X.......",
            snare: "....X.......X...",
            hat: "x.x.x.x.x.x.x.x.",
            open: "........x.......",
            extra: &[(4, "..o.....o.......")],
            bassline: "e2 ~ ~ g2 ~ ~ a2 ~ ~ ~ b2 ~ ~ e2 ~ ~",
        },
    ]
}

fn push_style(patterns: &mut Vec<Pattern>, songs: &mut Vec<Song>, s: Style) {
    let steps = if s.two_bar { 32 } else { 16 };
    let verse = pattern_from(&s, &format!("{} Verse", s.name), false);
    let mut chorus = verse.clone();
    chorus.name = format!("{} Chorus", s.name);
    chorus.drums.extend(row_hits(3, if s.two_bar { "........x...............x......." } else { "........x......." }, 0.7));
    chorus.drums.extend(row_hits(4, if s.two_bar { "......o.....................o..." } else { "......o........." }, 0.4));
    let intro = make_intro(&s, &verse);
    let fill = make_fill(&s);
    let brk = make_break(&s);
    let end = make_end(&s);
    let base = patterns.len();
    patterns.extend([intro, verse, chorus, fill, brk, end]);
    let intro_i = base;
    let verse_i = base + 1;
    let chorus_i = base + 2;
    let fill_i = base + 3;
    let break_i = base + 4;
    let end_i = base + 5;
    let verse_reps = if s.two_bar { 2 } else { 4 };
    let chorus_reps = if s.two_bar { 2 } else { 4 };
    let mut pads = vec![
        pad("INTRO", intro_i, PadJob::OnceThen { goto: 1 }),
        pad("VERSE", verse_i, PadJob::Loop),
        pad("CHORUS", chorus_i, PadJob::Loop),
        pad("FILL", fill_i, PadJob::Fill),
        pad("BREAK", break_i, PadJob::Fill),
        pad("END", end_i, PadJob::Stop),
    ];
    while pads.len() < PADS {
        pads.push(SongPad {
            label: "—".into(),
            pattern: verse_i,
            job: PadJob::Empty,
        });
    }
    songs.push(Song {
        name: s.name.into(),
        bpm: s.bpm,
        swing: s.swing,
        key: s.key,
        pads,
        arrangement: vec![
            ArrStep { pattern: intro_i, repeats: 1 },
            ArrStep { pattern: verse_i, repeats: verse_reps },
            ArrStep { pattern: fill_i, repeats: 1 },
            ArrStep { pattern: chorus_i, repeats: chorus_reps },
            ArrStep { pattern: break_i, repeats: 1 },
            ArrStep { pattern: verse_i, repeats: if s.two_bar { 1 } else { 2 } },
            ArrStep { pattern: fill_i, repeats: 1 },
            ArrStep { pattern: chorus_i, repeats: if s.two_bar { 1 } else { 2 } },
            ArrStep { pattern: end_i, repeats: 1 },
        ],
    });
    let _ = steps;
}

fn pad(label: &str, pattern: usize, job: PadJob) -> SongPad {
    SongPad {
        label: label.into(),
        pattern,
        job,
    }
}

fn pattern_from(s: &Style, name: &str, _chorus: bool) -> Pattern {
    let mut drums = Vec::new();
    drums.extend(row_hits(0, s.kick, 1.0));
    drums.extend(row_hits(1, s.snare, 1.0));
    drums.extend(row_hits(2, s.hat, 1.0));
    drums.extend(row_hits(3, s.open, 1.0));
    for (pad, row) in s.extra {
        drums.extend(row_hits(*pad, row, 1.0));
    }
    let bassline = parse_bass(s.bassline);
    let steps = step_count(&[s.kick, s.snare, s.hat, s.open]);
    Pattern {
        name: name.into(),
        kit: s.kit,
        bass: s.bass,
        bpm: s.bpm,
        swing: s.swing,
        key: s.key,
        steps,
        drums,
        bassline,
    }
}

fn make_intro(s: &Style, verse: &Pattern) -> Pattern {
    let half = verse.steps / 2;
    let mut drums: Vec<DrumHit> = verse
        .drums
        .iter()
        .cloned()
        .filter(|h| {
            let step = h.tick / 24;
            if step < half as u32 {
                matches!(h.pad, 2 | 3 | 5 | 6 | 8 | 10)
            } else {
                true
            }
        })
        .collect();
    drums.push(DrumHit { tick: 0, pad: 11, vel: 0.85 });
    let bassline = verse
        .bassline
        .iter()
        .cloned()
        .filter(|n| n.tick >= half as u32 * 24)
        .collect();
    Pattern {
        name: format!("{} Intro", s.name),
        drums,
        bassline,
        ..verse.clone()
    }
}

fn make_fill(s: &Style) -> Pattern {
    let steps: u16 = if s.two_bar { 16 } else { 16 };
    let mut drums = Vec::new();
    let seq = [7u8, 7, 8, 8, 9, 9, 1, 1];
    for (i, pad) in seq.iter().enumerate() {
        drums.push(DrumHit {
            tick: (i as u32) * 48,
            pad: *pad,
            vel: 0.55 + i as f32 * 0.06,
        });
    }
    drums.push(DrumHit { tick: 14 * 24, pad: 11, vel: 0.8 });
    drums.extend(row_hits(2, "x.x.x.x.x.x.x.x.", 0.45));
    let bassline = parse_bass(match s.key {
        _ => "c2 ~ ~ ~ g1 ~ ~ ~ c2 ~ ~ ~ c2 . c2 .",
    });
    // Bass is stored in C and transposed by pattern.key. Use the style root by
    // writing a C-based fill and letting key transpose it.
    let _ = s.key;
    Pattern {
        name: format!("{} Fill", s.name),
        kit: s.kit,
        bass: s.bass,
        bpm: s.bpm,
        swing: (s.swing * 0.35).min(0.2),
        key: s.key,
        steps,
        drums,
        bassline,
    }
}

fn make_break(s: &Style) -> Pattern {
    let steps: u16 = if s.two_bar { 32 } else { 16 };
    let mut drums = vec![DrumHit { tick: 0, pad: 0, vel: 0.75 }];
    let hat = if s.two_bar { "x.x.x.x.x.x.x.x. x.x.x.x.x.x.x.x." } else { "x.x.x.x.x.x.x.x." };
    drums.extend(row_hits(2, hat, 0.4));
    Pattern {
        name: format!("{} Break", s.name),
        kit: s.kit,
        bass: s.bass,
        bpm: s.bpm,
        swing: s.swing,
        key: s.key,
        steps,
        drums,
        bassline: Vec::new(),
    }
}

fn make_end(s: &Style) -> Pattern {
    let steps: u16 = if s.two_bar { 32 } else { 16 };
    let drums = vec![
        DrumHit { tick: 0, pad: 0, vel: 1.0 },
        DrumHit { tick: 0, pad: 1, vel: 0.9 },
        DrumHit { tick: 0, pad: 11, vel: 0.95 },
    ];
    let bassline = vec![BassNote { tick: 0, note: 36, dur: 96 * 2, vel: 0.9 }];
    Pattern {
        name: format!("{} End", s.name),
        kit: s.kit,
        bass: s.bass,
        bpm: s.bpm,
        swing: 0.0,
        key: s.key,
        steps,
        drums,
        bassline,
    }
}

fn step_count(rows: &[&str]) -> u16 {
    rows.iter().map(|r| count_steps(r)).max().unwrap_or(16).max(16) as u16
}

fn count_steps(row: &str) -> usize {
    row.chars().filter(|c| matches!(c, 'x' | 'X' | 'o' | '.')).count()
}

fn row_hits(pad: u8, row: &str, scale: f32) -> Vec<DrumHit> {
    let mut hits = Vec::new();
    let mut step = 0u32;
    for c in row.chars() {
        let vel = match c {
            'X' => 0.98,
            'x' => 0.62,
            'o' => 0.32,
            '.' => {
                step += 1;
                continue;
            }
            _ => continue,
        };
        hits.push(DrumHit { tick: step * 24, pad, vel: vel * scale });
        step += 1;
    }
    hits
}

fn parse_bass(line: &str) -> Vec<BassNote> {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    let mut notes = Vec::new();
    let mut i = 0usize;
    let mut cur: Option<(u8, usize, f32)> = None;
    let flush = |cur: &mut Option<(u8, usize, f32)>, i: usize, notes: &mut Vec<BassNote>| {
        if let Some((note, start, vel)) = cur.take() {
            let dur = i.saturating_sub(start).max(1);
            notes.push(BassNote {
                tick: (start as u32) * 24,
                note,
                dur: (dur as u32) * 24,
                vel,
            });
        }
    };
    for tok in tokens {
        if tok == "~" {
            i += 1;
            continue;
        }
        flush(&mut cur, i, &mut notes);
        if tok != "." {
            cur = Some((parse_note(tok), i, 0.9));
        }
        i += 1;
    }
    flush(&mut cur, i, &mut notes);
    notes
}

fn parse_note(s: &str) -> u8 {
    let s = s.trim().to_ascii_lowercase();
    let mut chars = s.chars();
    let letter = chars.next().unwrap_or('c');
    let mut semi = match letter {
        'c' => 0,
        'd' => 2,
        'e' => 4,
        'f' => 5,
        'g' => 7,
        'a' => 9,
        'b' => 11,
        _ => 0,
    };
    let rest: String = chars.collect();
    let oct_str = if rest.starts_with('#') || rest.starts_with('s') {
        semi += 1;
        &rest[1..]
    } else if let Some(stripped) = rest.strip_prefix('b') {
        semi -= 1;
        stripped
    } else {
        rest.as_str()
    };
    let oct: i32 = oct_str.parse().unwrap_or(2);
    (semi + (oct + 1) * 12).clamp(0, 96) as u8
}

fn kits() -> Vec<Kit> {
    let std = |name: &str, kick: u8, snare: u8, hat: u8, open: u8, tom: u8, ride: u8, crash: u8, clap: u8, decay: f32, send: f32| {
        kit(
            name,
            &[
                (kick, 0.0, 1.0, 0.0, decay, send * 0.4),
                (snare, 0.0, 0.92, 0.02, decay, send),
                (hat, 0.0, 0.42, 0.25, decay, send * 0.5),
                (open, 0.0, 0.5, 0.35, decay * 1.3, send * 0.8),
                (RIM, 0.0, 0.55, -0.1, 0.8, send * 0.3),
                (COWBELL, 0.0, 0.48, 0.4, 1.0, send * 0.4),
                (TAMB, 0.0, 0.46, -0.35, 1.0, send * 0.6),
                (tom, 4.0, 0.78, -0.25, decay, send * 0.5),
                (tom, 0.0, 0.8, 0.0, decay, send * 0.5),
                (tom, -5.0, 0.82, 0.28, decay * 1.1, send * 0.5),
                (ride, 0.0, 0.42, 0.45, 1.4, send * 0.7),
                (crash, 0.0, 0.55, -0.2, 1.6, send),
                (clap, 0.0, 0.7, 0.05, 1.0, send * 0.6),
            ],
        )
    };
    vec![
        std("Room Rock", KICK, SNARE, HAT, OPEN, TOM_M, RIDE, CRASH, CLAP, 1.05, 0.35),
        std("Tight Rock", KICK, SNARE, HAT, OPEN, TOM_H, RIDE, CRASH, CLAP, 0.75, 0.18),
        std("Funk Dry", KICK, SNARE, HAT, OPEN, TOM_H, RIDE, CRASH, CLAP, 0.7, 0.12),
        std("Boom Bap", KICK, SNARE, HAT, OPEN, TOM_M, RIDE, CRASH, CLAP, 1.15, 0.28),
        std("808", KICK808, SNARE808, HAT808, OPEN, TOM808, RIDE, CRASH, CLAP, 1.8, 0.2),
        std("909 House", KICK808, CLAP, HAT, OPEN, TOM808, RIDE, CRASH, CLAP, 1.1, 0.4),
        kit("Jazz Ride", &[
            (KICK, -1.0, 0.7, 0.0, 0.6, 0.2),
            (SNARE, 2.0, 0.45, 0.05, 0.55, 0.25),
            (BRUSH, 0.0, 0.35, 0.2, 0.8, 0.3),
            (BRUSH, 2.0, 0.4, 0.3, 1.2, 0.35),
            (STICK, 0.0, 0.5, -0.05, 0.5, 0.1),
            (COWBELL, 0.0, 0.3, 0.4, 0.8, 0.2),
            (BRUSH, -2.0, 0.3, -0.3, 0.9, 0.3),
            (TOM_H, 2.0, 0.55, -0.2, 0.7, 0.25),
            (TOM_M, 0.0, 0.55, 0.0, 0.75, 0.25),
            (TOM_L, -3.0, 0.58, 0.2, 0.85, 0.25),
            (RIDE, 0.0, 0.55, 0.35, 1.3, 0.45),
            (CRASH, 0.0, 0.4, -0.25, 1.5, 0.6),
            (RIDE_BELL, 0.0, 0.4, 0.15, 0.8, 0.3),
        ]),
        std("Disco", KICK, SNARE, HAT, OPEN, TOM_H, RIDE, CRASH, CLAP, 0.9, 0.42),
        kit("Reggae", &[
            (KICK, -2.0, 0.95, 0.0, 1.2, 0.3),
            (SNARE, -1.0, 0.7, 0.1, 0.8, 0.45),
            (HAT, 0.0, 0.4, -0.2, 0.9, 0.25),
            (OPEN, 0.0, 0.45, 0.3, 1.2, 0.4),
            (RIM, 0.0, 0.7, 0.15, 0.6, 0.2),
            (COWBELL, 0.0, 0.35, 0.4, 1.0, 0.2),
            (TAMB, 0.0, 0.3, -0.3, 1.0, 0.3),
            (TOM_H, 0.0, 0.6, -0.2, 1.0, 0.35),
            (TOM_M, -2.0, 0.65, 0.0, 1.1, 0.35),
            (TOM_L, -4.0, 0.7, 0.25, 1.2, 0.35),
            (RIDE, 0.0, 0.3, 0.4, 1.2, 0.4),
            (CRASH, 0.0, 0.4, -0.2, 1.4, 0.5),
            (CLAP, -2.0, 0.4, 0.0, 0.9, 0.3),
        ]),
        std("RnB", KICK, SNARE, HAT, OPEN, TOM_M, RIDE, CRASH, SNAP, 1.0, 0.33),
        std("Pop", KICK, SNARE, HAT, OPEN, TOM_H, RIDE, CRASH, CLAP, 0.85, 0.3),
        kit("DnB", &[
            (KICK808, -2.0, 1.05, 0.0, 0.9, 0.15),
            (SNARE, 1.0, 0.95, 0.05, 0.7, 0.2),
            (HAT, 1.0, 0.48, 0.3, 0.55, 0.15),
            (OPEN, 0.0, 0.5, 0.4, 0.8, 0.25),
            (RIM, 0.0, 0.6, -0.1, 0.5, 0.1),
            (COWBELL, 2.0, 0.3, 0.4, 0.6, 0.1),
            (SHAKER, 0.0, 0.35, -0.3, 0.6, 0.15),
            (TOM808, 3.0, 0.6, -0.2, 0.7, 0.15),
            (TOM808, 0.0, 0.65, 0.0, 0.8, 0.15),
            (TOM808, -4.0, 0.7, 0.25, 0.9, 0.15),
            (RIDE, 2.0, 0.35, 0.45, 0.8, 0.2),
            (CRASH, 0.0, 0.45, -0.3, 1.1, 0.35),
            (CLAP, 0.0, 0.55, 0.0, 0.6, 0.15),
        ]),
        kit("Afro", &[
            (KICK, 1.0, 0.95, 0.0, 0.85, 0.25),
            (SNARE, 2.0, 0.75, 0.05, 0.7, 0.3),
            (HAT, 0.0, 0.4, 0.2, 0.7, 0.25),
            (OPEN, 0.0, 0.48, 0.35, 1.0, 0.4),
            (RIM, 0.0, 0.65, -0.15, 0.6, 0.15),
            (COWBELL, 0.0, 0.55, 0.45, 1.0, 0.3),
            (SHAKER, 0.0, 0.5, -0.4, 0.8, 0.25),
            (CONGA, 5.0, 0.8, -0.3, 0.9, 0.3),
            (CONGA, 0.0, 0.85, 0.0, 1.0, 0.3),
            (CONGA, -5.0, 0.88, 0.32, 1.15, 0.3),
            (RIDE, 0.0, 0.35, 0.4, 1.1, 0.35),
            (CRASH, 0.0, 0.4, -0.2, 1.3, 0.5),
            (CLAP, 0.0, 0.45, 0.1, 0.7, 0.25),
        ]),
        kit("Brush", &[
            (KICK, 0.0, 0.55, 0.0, 0.7, 0.35),
            (BRUSH, 0.0, 0.55, 0.0, 0.9, 0.4),
            (BRUSH, 3.0, 0.32, 0.25, 0.6, 0.3),
            (BRUSH, 1.0, 0.4, 0.35, 1.1, 0.4),
            (STICK, 0.0, 0.4, -0.1, 0.5, 0.15),
            (COWBELL, -2.0, 0.25, 0.3, 0.8, 0.2),
            (SHAKER, 0.0, 0.28, -0.25, 0.9, 0.3),
            (TOM_H, 0.0, 0.45, -0.2, 0.8, 0.35),
            (TOM_M, -2.0, 0.48, 0.0, 0.9, 0.35),
            (TOM_L, -4.0, 0.5, 0.2, 1.0, 0.35),
            (RIDE, 0.0, 0.38, 0.3, 1.4, 0.5),
            (CRASH, -2.0, 0.35, -0.2, 1.6, 0.6),
            (SNAP, 0.0, 0.35, 0.05, 0.7, 0.25),
        ]),
        std("Electro", KICK808, SNARE808, HAT808, OPEN, TOM808, RIDE, CRASH, CLAP, 1.3, 0.22),
        std("Garage", KICK, SNARE, HAT, OPEN, TOM_M, RIDE, CRASH, CLAP, 0.8, 0.15),
        // Rock / blues kits for guitar jams. Appended so older kit indices stay stable.
        kit("Indie Room", &[
            (KICK_ROCK, -1.0, 1.0, 0.0, 0.95, 0.32),
            (SNARE_ROCK, 0.5, 0.95, 0.04, 0.95, 0.4),
            (HAT_ROCK, 0.0, 0.4, 0.22, 0.85, 0.22),
            (OPEN, -1.0, 0.48, 0.32, 1.15, 0.4),
            (RIM, 0.0, 0.55, -0.12, 0.7, 0.18),
            (COWBELL, -2.0, 0.28, 0.35, 0.8, 0.15),
            (TAMB, 0.0, 0.38, -0.3, 1.0, 0.35),
            (TOM_H, 2.0, 0.72, -0.22, 0.9, 0.3),
            (TOM_M, -1.0, 0.76, 0.0, 1.0, 0.3),
            (TOM_L, -5.0, 0.8, 0.26, 1.15, 0.32),
            (RIDE, 0.0, 0.4, 0.4, 1.35, 0.45),
            (CRASH, -1.0, 0.52, -0.18, 1.5, 0.55),
            (CLAP, 1.0, 0.45, 0.08, 0.8, 0.28),
        ]),
        kit("Alt Dry", &[
            (KICK_ROCK, 1.0, 1.05, 0.0, 0.7, 0.12),
            (SNARE_ROCK, 1.5, 1.0, 0.02, 0.7, 0.16),
            (HAT_ROCK, 1.0, 0.46, 0.28, 0.6, 0.1),
            (OPEN, 0.0, 0.5, 0.38, 0.9, 0.2),
            (RIM, 1.0, 0.62, -0.08, 0.55, 0.08),
            (STICK, 0.0, 0.4, 0.3, 0.55, 0.08),
            (SHAKER, 0.0, 0.32, -0.35, 0.7, 0.12),
            (TOM_H, 3.0, 0.7, -0.2, 0.65, 0.15),
            (TOM_M, 0.0, 0.74, 0.0, 0.75, 0.15),
            (TOM_L, -4.0, 0.78, 0.24, 0.85, 0.15),
            (RIDE, 2.0, 0.36, 0.42, 0.9, 0.18),
            (CRASH_ALT, 0.0, 0.58, -0.22, 1.0, 0.28),
            (SNAP, 0.0, 0.5, 0.05, 0.65, 0.12),
        ]),
        kit("Blues Shuffle", &[
            (KICK_ROCK, -2.0, 0.9, 0.0, 1.05, 0.38),
            (SNARE_ROCK, -1.0, 0.78, 0.05, 1.05, 0.42),
            (HAT_ROCK, -1.0, 0.38, 0.18, 0.95, 0.28),
            (OPEN, -2.0, 0.42, 0.28, 1.25, 0.45),
            (RIM, -1.0, 0.6, -0.1, 0.75, 0.2),
            (COWBELL, -3.0, 0.3, 0.35, 0.9, 0.18),
            (TAMB, -1.0, 0.34, -0.28, 1.05, 0.35),
            (TOM_H, 1.0, 0.65, -0.2, 0.95, 0.32),
            (TOM_M, -2.0, 0.7, 0.0, 1.05, 0.32),
            (TOM_L, -6.0, 0.74, 0.22, 1.2, 0.35),
            (RIDE, -1.0, 0.42, 0.38, 1.45, 0.5),
            (CRASH, -2.0, 0.45, -0.2, 1.6, 0.58),
            (CLAP, -1.0, 0.4, 0.05, 0.85, 0.3),
        ]),
    ]
}

fn kit(name: &str, spec: &[(u8, f32, f32, f32, f32, f32)]) -> Kit {
    Kit {
        name: name.into(),
        pads: spec
            .iter()
            .map(|(inst, tune, level, pan, decay, send)| PadAssign {
                inst: *inst,
                tune: *tune,
                level: *level,
                pan: *pan,
                decay: *decay,
                send: *send,
            })
            .collect(),
    }
}

fn basses() -> Vec<BassProgram> {
    let b = |name: &str, osc: u8, cutoff: f32, res: f32, sustain: f32, release: f32, punch: f32, glide: f32, sub: f32| {
        BassProgram {
            name: name.into(),
            osc,
            cutoff,
            res,
            sustain,
            release,
            punch,
            glide,
            sub,
        }
    };
    vec![
        b("Finger", 0, 420.0, 0.25, 0.72, 0.012, 0.25, 0.08, 0.18),
        b("Pick", 1, 680.0, 0.18, 0.55, 0.02, 0.45, 0.02, 0.08),
        b("Slap", 2, 900.0, 0.22, 0.42, 0.018, 0.85, 0.01, 0.12),
        b("Mute", 3, 500.0, 0.15, 0.15, 0.04, 0.55, 0.01, 0.05),
        b("Upright", 4, 280.0, 0.12, 0.35, 0.03, 0.3, 0.12, 0.22),
        b("Sub", 5, 180.0, 0.05, 0.9, 0.008, 0.1, 0.15, 0.0),
        b("Saw", 6, 740.0, 0.35, 0.8, 0.01, 0.2, 0.06, 0.15),
        b("Reese", 7, 520.0, 0.48, 0.85, 0.012, 0.15, 0.2, 0.25),
        b("Square", 8, 480.0, 0.4, 0.7, 0.014, 0.2, 0.05, 0.1),
        b("Fretless", 9, 360.0, 0.2, 0.78, 0.01, 0.15, 0.45, 0.2),
        b("Organ", 10, 640.0, 0.1, 0.9, 0.006, 0.05, 0.02, 0.05),
        b("Growl", 1, 320.0, 0.72, 0.8, 0.01, 0.35, 0.1, 0.3),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factory_is_consistent() {
        let doc = factory();
        assert!(doc.songs.len() >= 8);
        assert!(doc.patterns.len() >= doc.songs.len() * 6);
        assert!(doc.kits.len() >= 16);
        assert_eq!(doc.kits.len(), 19);
        assert_eq!(doc.basses.len(), 12);
        assert!(doc.songs.iter().any(|s| s.name == "Indie Rock"));
        assert!(doc.songs.iter().any(|s| s.name == "Alt Rock"));
        assert!(doc.songs.iter().any(|s| s.name == "Blues Shuffle"));
        assert!(doc.songs.iter().any(|s| s.name == "Rock"));
        assert!(doc.songs.iter().any(|s| s.name == "Blues"));
        for song in &doc.songs {
            assert_eq!(song.pads.len(), PADS);
            for step in &song.arrangement {
                assert!(step.pattern < doc.patterns.len());
            }
            for pad in &song.pads {
                assert!(pad.pattern < doc.patterns.len());
            }
        }
        for pattern in &doc.patterns {
            assert!(pattern.steps >= 16);
            assert!(pattern.kit < doc.kits.len());
            assert!(pattern.bass < doc.basses.len());
            for hit in &pattern.drums {
                assert!(hit.pad < PADS as u8);
                assert!(hit.vel > 0.0);
            }
        }
    }

    #[test]
    fn notes_parse() {
        assert_eq!(parse_note("c2"), 36);
        assert_eq!(parse_note("a2"), 45);
        assert_eq!(parse_note("f#2"), 42);
        assert_eq!(parse_note("eb2"), 39);
    }
}
