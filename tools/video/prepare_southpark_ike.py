"""Derive SFV1/Ogg from Ike's original MP4; FFmpeg is only needed offline.

Run: python tools/video/prepare_southpark_ike.py
Set FFMPEG_PATH or install imageio-ffmpeg. Originals are never rewritten.
"""
import hashlib
import argparse
import json
from pathlib import Path
import struct
import subprocess
import sys

sys.dont_write_bytecode = True
from prepare_smilingfriends_intro import ffmpeg_path, frames_in

ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "assets/video/south park/Kick_the_baby.mp4"
FPS = 24


def jpeg_size(data):
    offset = 2
    while offset < len(data):
        if data[offset] != 255:
            raise ValueError("Invalid JPEG marker")
        marker = data[offset + 1]
        size = int.from_bytes(data[offset + 2:offset + 4], "big")
        if marker in (0xC0, 0xC1, 0xC2):
            height, width = struct.unpack(">HH", data[offset + 5:offset + 9])
            return width, height
        offset += 2 + size
    raise ValueError("JPEG has no dimensions")


def check_derived():
    """Fail explicitly when the edited MP4 no longer matches the playable assets."""
    metadata = json.loads(SOURCE.with_name("kick_the_baby.json").read_text(encoding="utf-8"))
    if metadata["source_sha256"] != hashlib.sha256(SOURCE.read_bytes()).hexdigest():
        raise RuntimeError("Ike derivatives are stale: regenerate from the updated Kick_the_baby.mp4")
    for kind in ("video", "audio"):
        data = SOURCE.with_name(metadata[kind]).read_bytes()
        if metadata.get(kind + "_sha256") != hashlib.sha256(data).hexdigest():
            raise RuntimeError(f"Ike {kind} does not match the prepared metadata; regenerate")
    with SOURCE.with_name(metadata["video"]).open("rb") as stream:
        header = stream.read(20)
    expected = tuple(metadata[key] for key in ("width", "height", "fps", "frames"))
    if header[:4] != b"SFV1" or struct.unpack("<IIII", header[4:]) != expected:
        raise RuntimeError("Ike SFV dimensions/timing do not match metadata")
    print(f"IKE_DERIVED_OK source_sha256={metadata['source_sha256']} "
          f"size={metadata['width']}x{metadata['height']} fps={metadata['fps']}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="Verify source and derived hashes without FFmpeg or rewriting files")
    if parser.parse_args().check:
        check_derived()
        return
    exe = ffmpeg_path()
    result = subprocess.run(
        [exe, "-nostdin", "-hide_banner", "-loglevel", "error", "-i", str(SOURCE),
         "-map", "0:v:0", "-vf",
         f"setpts=PTS-STARTPTS,fps={FPS},scale=640:trunc(640/dar/2)*2:flags=lanczos,setsar=1",
         "-c:v", "mjpeg", "-q:v", "5", "-f", "image2pipe", "pipe:1"],
        stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=True)
    frames = list(frames_in(result.stdout))
    if not frames:
        raise RuntimeError("No frames decoded")
    width, height = jpeg_size(frames[0])
    packed = SOURCE.with_name("kick_the_baby.sfv.tmp")
    audio = SOURCE.with_name("kick_the_baby.ogg.tmp")
    with packed.open("wb") as stream:
        stream.write(b"SFV1" + struct.pack("<IIII", width, height, FPS, len(frames)))
        for frame in frames:
            stream.write(struct.pack("<I", len(frame)))
            stream.write(frame)
    subprocess.run(
        [exe, "-nostdin", "-hide_banner", "-loglevel", "error", "-y", "-i", str(SOURCE),
         "-map", "0:a:0", "-vn", "-af", "asetpts=PTS-STARTPTS", "-c:a", "libvorbis",
         "-q:a", "5", "-f", "ogg", str(audio)], check=True)
    packed.replace(SOURCE.with_name("kick_the_baby.sfv"))
    audio.replace(SOURCE.with_name("kick_the_baby.ogg"))
    metadata = dict(source=str(SOURCE.relative_to(ROOT)).replace("\\", "/"),
                    source_sha256=hashlib.sha256(SOURCE.read_bytes()).hexdigest(),
                    width=width, height=height, fps=FPS, frames=len(frames),
                    duration_seconds=len(frames) / FPS,
                    audio="kick_the_baby.ogg", video="kick_the_baby.sfv",
                    video_sha256=hashlib.sha256(SOURCE.with_name("kick_the_baby.sfv").read_bytes()).hexdigest(),
                    audio_sha256=hashlib.sha256(SOURCE.with_name("kick_the_baby.ogg").read_bytes()).hexdigest())
    SOURCE.with_name("kick_the_baby.json").write_text(json.dumps(metadata, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(metadata, indent=2))
    check_derived()


if __name__ == "__main__":
    main()
