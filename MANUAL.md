# Section manual

Section is a drum and bass rhythm machine for the browser and Ableton. The workflow follows a **Zoom RhythmTrak RT-223**: Song mode with intro / verse / chorus / fill / break / end on the pads, Pattern mode for editing, Groove for stacking loops, kits, bass programs, and a band you can mute when you only want drums.

Sounds, patterns, and names are original. This is not a Zoom product and not a sample dump of an RT-223.

Live page: https://www.pedro-muller.com/drum-machine/

## If you know an RT-223

| On the RT-223 | In Section |
| --- | --- |
| Song mode, pick a song, hit Intro / Verse / Fill / Ending pads | **Song**, pick a style, press sections **1–6** |
| Pattern mode, edit the grid / real-time record | **Pattern**, step grid, **Record** |
| Groove Play (layer patterns from pads) | **Groove** |
| Drum kits | **Kit** menu |
| Bass programs | **Bass** menu, or mute with **Bass off** / `\` |
| Tempo / swing | **Tempo**, **Swing** |
| Play pads while a song runs | Drum pads / keys **A S D F G H J** · **Z X C V B N** |

For guitar and bass practice, mute the synth bass and treat Song mode like the RT-223 accompaniment pads.

## Jam with guitar or bass (drums only)

1. Open the page and click **Play** once so the browser can start audio.
2. Click **Bass on** so it becomes **Bass off**, or press `\`. The synth bass stops. The drums keep going.
3. Pick a style in the song row. For guitar jams start with **Rock**, **Hard Rock**, **Indie Rock**, **Alt Rock**, **Blues**, or **Blues Shuffle**. The older styles (Funk, House, Jazz, …) are still there.
4. Set the tempo with **Tempo − / +**, or `[` and `]`, until it matches the song you want to play.
5. Stay in **Song**. Press **Space** or **Play**, or hit a section pad.
6. Build the form live with sections **1–6**, the same idea as the RT-223 song pads:
   - **1 Intro** — plays once, then goes to the verse
   - **2 Verse** — loops
   - **3 Chorus** — loops
   - **4 Fill** — fill, then back to the latched loop
   - **5 Break** — break, then back
   - **6 End** — ending, then stop
7. Hit the drum pads while it plays for live accents. Keys: **A S D F G H J** and **Z X C V B N**.
8. Press **Space** or **Stop** when you are done.

Leave the synth bass muted. Your bass and guitar fill the low end.

Tip: if the feel is too stiff, raise **Swing**. For rock / indie / blues, try kits **Indie Room**, **Alt Dry**, or **Blues Shuffle** — the original kits stay in the list. Leave **Key** alone when the bass is muted; key mainly moves the synth bass.

## What the modes mean

| Mode | RT-223 idea | Job in Section |
| --- | --- | --- |
| **Song** | Song mode + pad-assembled form | Play a style. Section buttons change intro / verse / chorus / fill / break / end. Use this to jam. |
| **Pattern** | Pattern mode | Edit one pattern. The step grid and Record write into that pattern. |
| **Groove** | Groove Play | Layer section patterns from the pads. |

Start in **Song**. Switch to **Pattern** only when you want to rewrite the drums.

## Transport and mute

| Control | What it does |
| --- | --- |
| **Play / Stop** or **Space** | Start or stop the band |
| **Record** or **R** | Arm recording into the pattern you are hearing. Section switches to Pattern mode for that. |
| **Bass on / Bass off** or `\` | Mute or unmute the synth bass |
| **Click** or **M** | Metronome |
| **Drums / Bass** or **Tab** | Pads play drums, or pads play the synth bass notes |

## Pads and the grid

There are 13 drum pads: Kick, Snare, Hat, Open, Rim, Bell, Tamb, Hi Tom, Mid Tom, Lo Tom, Ride, Crash, Clap.

- Click a pad, or hold its key, to play it.
- Hit higher on the pad for a softer velocity; lower on the pad for a harder hit.
- The step grid under the pads is the current pattern. Click a cell to toggle a drum hit.
- The bottom **BASS** row is the synth bassline. Click a step to place or clear a note. Right-click raises the note. Mute the bass when you are playing your own.

## Make a groove your own

1. Pick a style close to the song.
2. Mute the synth bass if you are accompanying yourself.
3. Press **Play**, then **Record**.
4. Play pads over the pattern. Hits land on the current step (16th-note quantize by default).
5. Or stop, open the grid, and click steps in.
6. Press **Save** to keep the document in this browser. **Factory** throws away local edits and restores the stock songs.

**Duplicate** copies the current pattern so you can keep the original and edit the copy. **Clear drums** / **Clear bass** wipe that part of the current pattern.

## Compose a longer song

Like the RT-223, a “song” here is a style with section pads and an arrangement. For a practice song with guitar and bass:

1. Choose the style that matches the feel.
2. Mute the synth bass.
3. Set tempo.
4. Use **1–6** live as the form: Intro → Verse → Fill → Chorus → Break → Verse → End.
5. If a part needs different drums, switch to **Pattern**, pick that pattern in the Pattern menu, edit the grid, then go back to **Song**.

You do not need to write a full arrangement file for jamming. The section pads are the form.

## Keyboard map

| Keys | Action |
| --- | --- |
| **Space** | Play / stop |
| **1–6** | Section pads (Intro … End) |
| **A S D F G H J** | First seven drum pads |
| **Z X C V B N** | Last six drum pads |
| **[ ]** | Tempo − / + |
| **− =** | Key − / + (synth bass) |
| **Tab** | Drums / bass pads |
| **\\** | Mute synth bass |
| **R** | Record |
| **M** | Metronome |

## Ableton

Install the VST3 from the Section repo (`BUILD.md`). Load **section** on a MIDI track.

- MIDI channel **10** plays the drum kit.
- Any other channel plays the synth bass.
- When Live’s transport runs, Section follows the host tempo and position.
- Preview with Play in the plugin when Live is stopped.

## If something is wrong

- No sound: click the page, then Play or a pad once. The browser needs a gesture before audio starts.
- Too much bass under your guitar: mute the synth bass.
- Wrong feel: change style, tempo, swing, or kit before rewriting the grid.
- Edits vanished after a refresh: press **Save** next time. **Factory** clears local saves on purpose.
