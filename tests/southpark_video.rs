use planet_hoppers::scenes::southpark_video::{skip_button, video_rect, VideoFrames, VIDEO};
use std::path::Path;

fn pack(width: u32, height: u32, fps: u32, frames: &[&[u8]]) -> Vec<u8> {
    let mut bytes = b"SFV1".to_vec();
    for value in [width, height, fps, frames.len() as u32] {
        bytes.extend(value.to_le_bytes());
    }
    for frame in frames {
        bytes.extend((frame.len() as u32).to_le_bytes());
        bytes.extend(*frame);
    }
    bytes
}

#[test]
fn rejects_truncation_invalid_metadata_and_trailing_data() {
    let valid = pack(1280, 720, 30, &[b"jpeg", b"jpeg2"]);
    for length in 0..valid.len() {
        assert!(VideoFrames::from_bytes(valid[..length].to_vec()).is_err());
    }
    for (width, height, fps) in [(0, 720, 30), (1280, 0, 30), (1280, 720, 0), (1280, 720, 61)] {
        assert!(VideoFrames::from_bytes(pack(width, height, fps, &[b"jpeg"])).is_err());
    }
    assert!(VideoFrames::from_bytes(pack(1280, 720, 30, &[])).is_err());
    assert!(VideoFrames::from_bytes(pack(1280, 720, 30, &[b""])).is_err());
    let mut trailing = valid;
    trailing.push(0);
    assert!(VideoFrames::from_bytes(trailing).is_err());
}

#[test]
fn indexed_file_reader_validates_lengths_and_matches_memory_reader() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/video-index-test");
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("validation.sfv");
    let valid = pack(16, 9, 24, &[b"jpeg", b"jpeg2"]);
    for length in 0..valid.len() {
        std::fs::write(&path, &valid[..length]).unwrap();
        assert!(VideoFrames::load(&path).is_err());
    }
    std::fs::write(&path, &valid).unwrap();
    let disk = VideoFrames::load(&path).unwrap();
    let memory = VideoFrames::from_bytes(valid.clone()).unwrap();
    assert_eq!(disk.duration(), memory.duration());
    assert_eq!(disk.frame_at(1.0), memory.frame_at(1.0));
    drop(disk);
    let mut trailing = valid;
    trailing.push(0);
    std::fs::write(&path, trailing).unwrap();
    assert!(VideoFrames::load(&path).is_err());
}

#[test]
fn frame_selection_follows_the_stream_clock_and_clamps_at_the_end() {
    let frames = VideoFrames::from_bytes(pack(16, 9, 30, &[b"a", b"b", b"c"])).unwrap();
    assert_eq!(frames.frame_at(-1.0), 0);
    assert_eq!(frames.frame_at(0.034), 1);
    assert_eq!(frames.frame_at(0.067), 2);
    assert_eq!(frames.frame_at(100.0), 2);
    assert!((frames.duration() - 0.1).abs() < 0.0001);
}

#[test]
fn real_clip_matches_prepared_metadata_and_expected_duration() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let frames = VideoFrames::load(&root.join(VIDEO)).unwrap();
    assert_eq!((frames.width, frames.height, frames.fps), (1280, 720, 30));
    assert!((frames.duration() - 26.7).abs() < 0.001);
    assert_eq!(frames.frame_at(frames.duration()), 800);
    let metadata: serde_json::Value = serde_json::from_slice(
        &std::fs::read(root.join("assets/video/south park/intro.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(metadata["frames"], 801);
    assert_eq!(
        metadata["source"],
        "assets/video/south park/South_park_intro.mp4"
    );
}

#[test]
fn video_keeps_aspect_ratio_and_skip_never_covers_it() {
    for (w, h) in [
        (640, 480),
        (960, 640),
        (1280, 800),
        (1920, 1080),
        (800, 1000),
    ] {
        for (source_w, source_h) in [(1920, 1080), (640, 480)] {
            let video = video_rect(w, h, source_w, source_h);
            let skip = skip_button(w, h);
            assert!((video.width / video.height - source_w as f32 / source_h as f32).abs() < 0.001);
            assert!(video.x > 0.0 && video.y > 0.0);
            assert!(video.x + video.width < w as f32);
            assert!(video.y + video.height + 20.0 < skip.y);
            assert!(skip.x > 0.0 && skip.x + skip.width < w as f32);
            assert!(skip.y + skip.height < h as f32);
        }
    }
}
