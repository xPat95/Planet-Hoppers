"""Prepare the supplied MP4 for raylib. FFmpeg is needed only for this step.

Uses the project's SFV1 (JPEG frames) + streaming Ogg convention. The original
MP4 stays untouched. Run from any directory; set FFMPEG_PATH if necessary.
"""
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import sys

sys.dont_write_bytecode = True
from prepare_smilingfriends_intro import ffmpeg_path, frames_in

ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "assets/video/south park/South_park_intro.mp4"
DEST = SOURCE.parent
WIDTH, HEIGHT, FPS = 1280, 720, 30


def main():
    if not SOURCE.is_file():
        raise FileNotFoundError(SOURCE)
    exe = ffmpeg_path()
    result = subprocess.run(
        [exe, "-nostdin", "-hide_banner", "-loglevel", "error", "-i", str(SOURCE),
         "-map", "0:v:0", "-vf", f"fps={FPS},scale={WIDTH}:{HEIGHT}:flags=lanczos,setsar=1",
         "-c:v", "mjpeg", "-q:v", "5", "-f", "image2pipe", "pipe:1"],
        stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=True,
    )
    frames = list(frames_in(result.stdout))
    if not frames:
        raise RuntimeError("No video frames decoded")
    # Publish only complete files, so an interrupted conversion is not playable.
    packed = DEST / "intro.sfv.tmp"
    with packed.open("wb") as stream:
        stream.write(b"SFV1" + struct.pack("<IIII", WIDTH, HEIGHT, FPS, len(frames)))
        for frame in frames:
            stream.write(struct.pack("<I", len(frame)))
            stream.write(frame)
    audio = DEST / "intro.ogg.tmp"
    subprocess.run(
        [exe, "-nostdin", "-hide_banner", "-loglevel", "error", "-y", "-i", str(SOURCE),
         "-map", "0:a:0", "-vn", "-c:a", "libvorbis", "-q:a", "5", "-f", "ogg", str(audio)],
        check=True,
    )
    packed.replace(DEST / "intro.sfv")
    audio.replace(DEST / "intro.ogg")
    metadata = dict(source=str(SOURCE.relative_to(ROOT)).replace("\\", "/"),
                    source_sha256=hashlib.sha256(SOURCE.read_bytes()).hexdigest(),
                    width=WIDTH, height=HEIGHT, fps=FPS, frames=len(frames),
                    duration_seconds=len(frames) / FPS,
                    audio="intro.ogg", video="intro.sfv")
    (DEST / "intro.json").write_text(json.dumps(metadata, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(metadata, indent=2), flush=True)


if __name__ == "__main__":
    main()
