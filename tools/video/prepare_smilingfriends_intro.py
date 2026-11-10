"""Pack the supplied MOV as JPEG frames plus streaming Ogg audio for raylib.

Run from the repository root: `python tools/video/prepare_smilingfriends_intro.py [intro|abduction]` (default: intro).
`source/<name>.mov` becomes `<name>.sfv` and `<name>.ogg`. The abduction is shown full screen, so it is packed larger. Requires ffmpeg on PATH or imageio-ffmpeg installed
in the Python used to run this script. The game does not need ffmpeg at runtime.
"""

import os
import sys
from pathlib import Path
import shutil
import struct
import subprocess

ROOT = Path(__file__).resolve().parents[2]
DEST = ROOT / "assets" / "video" / "smilingfriends"
SIZES = {"intro": (960, 540), "abduction": (1280, 720)}
FPS = 24


def ffmpeg_path():
    path = os.environ.get("FFMPEG_PATH") or shutil.which("ffmpeg")
    if path:
        return path
    try:
        import imageio_ffmpeg

        return imageio_ffmpeg.get_ffmpeg_exe()
    except ImportError as exc:
        raise RuntimeError("Install ffmpeg or `python -m pip install imageio-ffmpeg`") from exc


def frames_in(stream):
    pos = 0
    while True:
        start = stream.find(b"\xff\xd8", pos)
        if start < 0:
            if stream[pos:].strip():
                raise RuntimeError("Unexpected bytes after last JPEG frame")
            return
        end = stream.find(b"\xff\xd9", start + 2)
        if end < 0:
            raise RuntimeError("Incomplete JPEG frame")
        end += 2
        yield stream[start:end]
        pos = end


def main():
    name = sys.argv[1] if len(sys.argv) > 1 else "intro"
    if name not in SIZES:
        raise SystemExit(f"Unknown video: {name} (expected one of {', '.join(SIZES)})")
    WIDTH, HEIGHT = SIZES[name]
    SOURCE = DEST / "source" / f"{name}.mov"
    if not SOURCE.is_file():
        raise FileNotFoundError(SOURCE)
    exe = ffmpeg_path()
    DEST.mkdir(parents=True, exist_ok=True)
    result = subprocess.run(
        [exe, "-nostdin", "-hide_banner", "-loglevel", "error", "-i", str(SOURCE),
         "-map", "0:v:0", "-vf", f"fps={FPS},scale={WIDTH}:{HEIGHT}:flags=lanczos",
         "-c:v", "mjpeg", "-q:v", "7", "-f", "image2pipe", "pipe:1"],
        stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=True,
    )
    frames = list(frames_in(result.stdout))
    if not frames:
        raise RuntimeError("No video frames decoded")
    out = DEST / f"{name}.sfv"
    with out.open("wb") as f:
        f.write(b"SFV1")
        f.write(struct.pack("<IIII", WIDTH, HEIGHT, FPS, len(frames)))
        for frame in frames:
            f.write(struct.pack("<I", len(frame)))
            f.write(frame)
    audio = DEST / f"{name}.ogg"
    subprocess.run(
        [exe, "-nostdin", "-hide_banner", "-loglevel", "error", "-y", "-i", str(SOURCE),
         "-map", "0:a:0", "-vn", "-c:a", "libvorbis", "-q:a", "4", str(audio)],
        check=True,
    )
    print(f"{len(frames)} frames at {FPS} fps ({len(frames) / FPS:.2f} s): {out}")
    print(f"Audio: {audio}")


if __name__ == "__main__":
    main()
