use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn workspace_root() -> PathBuf {
    // crates/svt-av1 -> crates -> <workspace root>
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("unexpected svt-av1 path layout")
        .to_path_buf()
}

fn unique_test_dir(prefix: &str) -> PathBuf {
    let ws = workspace_root();
    let pid = std::process::id();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();

    let dir = ws
        .join("target")
        .join("e2e")
        .join(format!("{prefix}-{pid}-{nanos}"));
    fs::create_dir_all(&dir).expect("create e2e dir");
    dir
}

fn write_yuv420p8(path: &Path, width: u32, height: u32, frames: usize) {
    assert_eq!(width % 2, 0, "yuv420 requires even width");
    assert_eq!(height % 2, 0, "yuv420 requires even height");

    let luma_len = (width as usize) * (height as usize);
    let chroma_len = luma_len / 4;
    let frame_size = luma_len + chroma_len * 2;

    let mut buf = vec![0u8; frame_size];

    // Simple deterministic frame: neutral chroma, luma varies per frame.
    buf[luma_len..luma_len + chroma_len].fill(128);
    buf[luma_len + chroma_len..].fill(128);

    let mut f = fs::File::create(path).expect("create yuv input");
    for i in 0..frames {
        buf[..luma_len].fill((16 + (i as u8).saturating_mul(32)) as u8);
        f.write_all(&buf).expect("write yuv frame");
    }
}

#[derive(Debug)]
struct IvfHeader {
    fourcc: [u8; 4],
    width: u16,
    height: u16,
    fps_num: u32,
    fps_den: u32,
    frame_count: u32,
}

fn parse_ivf_header(data: &[u8]) -> IvfHeader {
    assert!(data.len() >= 32, "ivf too short");
    assert_eq!(&data[0..4], b"DKIF", "missing DKIF header");
    let fourcc = data[8..12].try_into().expect("fourcc");
    let width = u16::from_le_bytes([data[12], data[13]]);
    let height = u16::from_le_bytes([data[14], data[15]]);
    let fps_num = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);
    let fps_den = u32::from_le_bytes([data[20], data[21], data[22], data[23]]);
    let frame_count = u32::from_le_bytes([data[24], data[25], data[26], data[27]]);
    IvfHeader {
        fourcc,
        width,
        height,
        fps_num,
        fps_den,
        frame_count,
    }
}

fn count_ivf_frames(data: &[u8]) -> u32 {
    if data.len() < 32 {
        return 0;
    }
    let mut offset = 32usize;
    let mut frames = 0u32;
    while offset + 12 <= data.len() {
        let size = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as usize;
        offset += 12;
        if offset + size > data.len() {
            break;
        }
        frames += 1;
        offset += size;
    }
    frames
}

fn parse_ivf_frames(data: &[u8]) -> Vec<(u32, u64)> {
    let mut frames = Vec::new();
    if data.len() < 32 {
        return frames;
    }

    let mut offset = 32usize;
    while offset + 12 <= data.len() {
        let size = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        let ts = u64::from_le_bytes([
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
            data[offset + 8],
            data[offset + 9],
            data[offset + 10],
            data[offset + 11],
        ]);
        offset += 12;

        let size_usize = size as usize;
        if offset + size_usize > data.len() {
            break;
        }
        frames.push((size, ts));
        offset += size_usize;
    }

    frames
}

fn assert_success(out: &Output, what: &str) {
    if !out.status.success() {
        panic!(
            "{what} failed (exit={}).\n--- stdout ---\n{}\n--- stderr ---\n{}\n",
            out.status,
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

#[test]
fn e2e_cli_encode_writes_valid_ivf() {
    let dir = unique_test_dir("encode");
    let yuv_path = dir.join("in.yuv");
    let ivf_path = dir.join("out.ivf");

    // Keep the encode short; we only need enough output to validate the container.
    let width = 64u32;
    let height = 64u32;
    write_yuv420p8(&yuv_path, width, height, 1);

    let mut cmd = Command::new("cargo");
    cmd.current_dir(workspace_root())
        .args([
            "run",
            "-q",
            "-p",
            "svt-av1",
            "--example",
            "encode",
            "--",
            &width.to_string(),
            &height.to_string(),
            yuv_path.to_str().expect("yuv path"),
            ivf_path.to_str().expect("ivf path"),
        ])
        // Make the test hermetic regardless of the developer's environment.
        .env("SVT_AV1_BUILD_FROM_SOURCE", "1")
        .env("SVT_AV1_NO_PKG_CONFIG", "1")
        .env("SVT_AV1_INCLUDE_DIR", "vendor/SVT-AV1/Source/API")
        .env_remove("SVT_AV1_LIB_DIR")
        .env_remove("SVT_AV1_PKG_CONFIG_NAME")
        .env("CARGO_TERM_COLOR", "never");
    let out = cmd.output().expect("run encode example");
    assert_success(&out, "encode example");

    let data = fs::read(&ivf_path).expect("read ivf output");
    let hdr = parse_ivf_header(&data);
    assert_eq!(&hdr.fourcc, b"AV01", "unexpected fourcc");
    assert_eq!(hdr.width, width as u16);
    assert_eq!(hdr.height, height as u16);
    assert_eq!((hdr.fps_num, hdr.fps_den), (30, 1));

    let frame_count = count_ivf_frames(&data);
    assert!(frame_count > 0, "expected at least one IVF frame");
    assert_eq!(
        hdr.frame_count, frame_count,
        "IVF header frame count should match payload"
    );
}

#[test]
fn e2e_cli_encode_multiple_frames_writes_expected_timestamps() {
    let dir = unique_test_dir("encode-multi");
    let yuv_path = dir.join("in.yuv");
    let ivf_path = dir.join("out.ivf");

    let width = 64u32;
    let height = 64u32;
    let frames_in = 3usize;
    write_yuv420p8(&yuv_path, width, height, frames_in);

    let mut cmd = Command::new("cargo");
    cmd.current_dir(workspace_root())
        .args([
            "run",
            "-q",
            "-p",
            "svt-av1",
            "--example",
            "encode",
            "--",
            &width.to_string(),
            &height.to_string(),
            yuv_path.to_str().expect("yuv path"),
            ivf_path.to_str().expect("ivf path"),
        ])
        .env("SVT_AV1_BUILD_FROM_SOURCE", "1")
        .env("SVT_AV1_NO_PKG_CONFIG", "1")
        .env("SVT_AV1_INCLUDE_DIR", "vendor/SVT-AV1/Source/API")
        .env_remove("SVT_AV1_LIB_DIR")
        .env_remove("SVT_AV1_PKG_CONFIG_NAME")
        .env("CARGO_TERM_COLOR", "never");
    let out = cmd.output().expect("run encode example");
    assert_success(&out, "encode example");

    let data = fs::read(&ivf_path).expect("read ivf output");
    let hdr = parse_ivf_header(&data);
    assert_eq!(hdr.width, width as u16);
    assert_eq!(hdr.height, height as u16);

    let frames = parse_ivf_frames(&data);
    assert_eq!(
        frames.len(),
        frames_in,
        "expected one IVF packet per input frame"
    );
    assert_eq!(hdr.frame_count as usize, frames_in);
    assert!(frames.iter().all(|(size, _)| *size > 0));

    let mut timestamps: Vec<u64> = frames.iter().map(|(_, ts)| *ts).collect();
    timestamps.sort_unstable();
    assert_eq!(timestamps, vec![0, 1, 2], "unexpected packet timestamps");
}

#[test]
fn e2e_cli_encode_then_decode_roundtrip() {
    // Decoder is not available from the vendored SVT-AV1 copy; this test
    // requires a decoder-capable system install. Keep it opt-in.
    if std::env::var_os("SVT_AV1_E2E_DECODER").is_none() {
        eprintln!("skipping decode e2e: set SVT_AV1_E2E_DECODER=1 to enable");
        return;
    }

    let dir = unique_test_dir("encode-decode");
    let yuv_path = dir.join("in.yuv");
    let ivf_path = dir.join("out.ivf");

    let width = 64u32;
    let height = 64u32;
    write_yuv420p8(&yuv_path, width, height, 1);

    // Encode using the repo's program.
    let mut enc = Command::new("cargo");
    enc.current_dir(workspace_root())
        .args([
            "run",
            "-q",
            "-p",
            "svt-av1",
            "--example",
            "encode",
            "--",
            &width.to_string(),
            &height.to_string(),
            yuv_path.to_str().expect("yuv path"),
            ivf_path.to_str().expect("ivf path"),
        ])
        .env("SVT_AV1_BUILD_FROM_SOURCE", "1")
        .env("SVT_AV1_NO_PKG_CONFIG", "1")
        .env("SVT_AV1_INCLUDE_DIR", "vendor/SVT-AV1/Source/API")
        .env_remove("SVT_AV1_LIB_DIR")
        .env_remove("SVT_AV1_PKG_CONFIG_NAME")
        .env("CARGO_TERM_COLOR", "never");
    let out = enc.output().expect("run encode example");
    assert_success(&out, "encode example");

    // Decode using the repo's program.
    let mut dec = Command::new("cargo");
    dec.current_dir(workspace_root())
        .args([
            "run",
            "-q",
            "-p",
            "svt-av1",
            "--features",
            "decoder",
            "--example",
            "decode",
            "--",
            ivf_path.to_str().expect("ivf path"),
        ])
        .env("SVT_AV1_NO_PKG_CONFIG", "0")
        // Ensure we don't accidentally force a vendored build (encoder-only).
        .env("SVT_AV1_BUILD_FROM_SOURCE", "0")
        .env("CARGO_TERM_COLOR", "never");

    let out = dec.output().expect("run decode example");
    assert_success(&out, "decode example");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let decoded = stdout
        .split("Decoded ")
        .nth(1)
        .and_then(|s| s.split_whitespace().next())
        .and_then(|n| n.parse::<u32>().ok())
        .unwrap_or(0);
    assert!(decoded > 0, "expected decoded frames > 0, got:\n{stdout}");
}
