use svt_av1::encoder::{Encoder, Configuration, BufferHeader};
use svt_av1::config::{ConfigExt, BitDepth, ColorFormat, Profile, Tier, RcMode, IntraRefreshType};

// This example demonstrates initializing the encoder, tweaking a couple of
// configuration values, retrieving stream headers, and draining packets.
//
// It is meant to compile in CI using vendored headers (no linking required):
//   SVT_AV1_NO_PKG_CONFIG=1 SVT_AV1_INCLUDE_DIR=vendor/SVT-AV1/Source/API \
//   cargo check -p svt-av1 --example encode

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Print version from the library (static string)
    let ver = Encoder::version();
    eprintln!("SVT-AV1 version: {}", ver.to_string_lossy());
    Encoder::print_version();

    // Create encoder and default configuration
    let (mut enc, mut cfg): (Encoder, Configuration) = Encoder::init_default()?;

    // Set a few minimal parameters using typed helpers
    cfg.set_resolution(320, 240)
        .set_frame_rate(30, 1)
        .set_bit_depth(BitDepth::Eight)
        .set_color_format(ColorFormat::Yuv420)
        .set_profile(Profile::Main)
        .set_tier(Tier::Main)
        .set_level_auto()
        .set_rc_mode(RcMode::Vbr)
        .set_qp(50)
        .set_intra_refresh(IntraRefreshType::FwdKey);
    cfg.intra_period_length = 30; // simple GOP

    // Alternatively, set via name/value parser (demonstration)
    Encoder::parse_parameter_str(&mut cfg, "rc", "vbr").ok();

    enc.set_parameter(&cfg)?;
    enc.init()?;

    // Retrieve stream header (codec config) if needed
    let mut header_pkt: *mut BufferHeader = std::ptr::null_mut();
    enc.get_stream_header(&mut header_pkt)?;
    if !header_pkt.is_null() {
        // Process stream header here (contained in header_pkt)
        // Release when done
        enc.stream_header_release(header_pkt)?;
    }

    // No frames are sent in this minimal example. After finishing input, set pic_send_done.
    let pic_send_done = true;

    // Option A: drain with callback
    enc.drain_packets(pic_send_done, |pkt| {
        let size = pkt.n_filled_len;
        let pts = pkt.pts;
        eprintln!("got packet: {} bytes, pts={}", size, pts);
    })?;

    // Option B: iterator with RAII packet wrapper
    for pkt in enc.packets(pic_send_done) {
        let pkt = pkt?;
        let hdr = pkt.header();
        eprintln!("iter packet: {} bytes, flags={}", hdr.n_filled_len, hdr.flags);
        // Dropping `pkt` releases it back to SVT-AV1
    }

    Ok(())
}
