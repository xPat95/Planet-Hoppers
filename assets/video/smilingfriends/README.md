# Smiling Friends opening video

The first screen shown when entering the Smiling Friends planet is the video in `source/intro.mov` (packed by `tools/video/prepare_smilingfriends_intro.py`). It plays once with its original audio, then shows a temporary **En construcción** screen. Enter or Esc returns to the solar system; Space, Enter, or Esc skips the video.

Raylib does not decode MOV during gameplay. The generated `intro.sfv` contains 851 JPEG frames at 960×540 and 24 fps; `intro.ogg` contains the audio. The player reads one frame at a time and reuses a single GPU texture. No external video program is needed to run the game.

To regenerate these assets after replacing the MOV, install FFmpeg or `imageio-ffmpeg`, then run:

```powershell
python tools/video/prepare_smilingfriends_intro.py
```

To check the first frame, last frame, audio length, and construction screen without waiting for the whole clip:

```powershell
cargo run --locked --bin smilingfriends -- --smoke
```

The test saves screenshots under `target/smilingfriends/`.

## Abduction video

`source/abduction.mov` is the Smiling Friends abduction, shown full screen when the Smiling Friends ship is picked in the hangar (see `docs/minigames/abduction-rewards.md`). It is packed the same way, at 1280×720 and 24 fps (311 frames), into `abduction.sfv` and `abduction.ogg`:

```powershell
python tools/video/prepare_smilingfriends_intro.py abduction
```

If the clip changes length, update `SMILINGFRIENDS_ABDUCTION_SECONDS` in `src/scenes/token_collection.rs` (a test checks that they match).
