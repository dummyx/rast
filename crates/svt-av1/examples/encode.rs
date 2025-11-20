use std::fs::File;
use std::io::BufWriter;
use std::io::{Read, Seek, SeekFrom, Write};

struct IvfWriter<W: Write + Seek> {
    inner: W,
    frame_count_pos: u64,
    frame_count: u32,
}

impl<W: Write + Seek> IvfWriter<W> {
    fn new(
        mut inner: W,
        width: u16,
        height: u16,
        fps_num: u32,
        fps_den: u32,
    ) -> std::io::Result<Self> {
        // IVF header (32 bytes)
        // 0-3  : DKIF
        // 4-5  : version (0)
        // 6-7  : header size (32)
        // 8-11 : fourcc "AV01"
        // 12-13: width (LE)
        // 14-15: height (LE)
        // 16-19: framerate (LE)
        // 20-23: timescale (LE)
        // 24-27: frame count (LE, 0 if unknown)
        // 28-31: reserved
        inner.write_all(b"DKIF")?;
        inner.write_all(&0u16.to_le_bytes())?;
        inner.write_all(&32u16.to_le_bytes())?;
        inner.write_all(b"AV01")?;
        inner.write_all(&width.to_le_bytes())?;
        inner.write_all(&height.to_le_bytes())?;
        inner.write_all(&fps_num.to_le_bytes())?;
        inner.write_all(&fps_den.to_le_bytes())?;
        let frame_count_pos = inner.stream_position()?;
        inner.write_all(&0u32.to_le_bytes())?;
        inner.write_all(&0u32.to_le_bytes())?;
        Ok(Self {
            inner,
            frame_count_pos,
            frame_count: 0,
        })
    }

    fn write_frame(&mut self, data: &[u8], timestamp: u64) -> std::io::Result<()> {
        self.inner.write_all(&(data.len() as u32).to_le_bytes())?;
        self.inner.write_all(&timestamp.to_le_bytes())?;
        self.inner.write_all(data)?;
        self.frame_count = self.frame_count.wrapping_add(1);
        Ok(())
    }
}

impl<W: Write + Seek> Drop for IvfWriter<W> {
    fn drop(&mut self) {
        if self
            .inner
            .seek(SeekFrom::Start(self.frame_count_pos))
            .is_ok()
        {
            let _ = self.inner.write_all(&self.frame_count.to_le_bytes());
            let _ = self.inner.flush();
        }
    }
}
use svt_av1::config::{BitDepth, ColorFormat, ConfigExt, IntraRefreshType, Profile, RcMode, Tier};
use svt_av1::encoder::{BufferHeader, Configuration, Encoder};
use svt_av1_sys as sys;

// This example demonstrates initializing the encoder, tweaking a couple of
// configuration values, retrieving stream headers, and draining packets.
//
// If a raw 8-bit 4:2:0 YUV file is provided on the command line
// as: width height path.yuv, the example will also read frames from
// that file, send them into the encoder, and drain the resulting packets.
//
// It is meant to compile in CI using vendored headers (no linking required):
//   SVT_AV1_NO_PKG_CONFIG=1 SVT_AV1_INCLUDE_DIR=vendor/SVT-AV1/Source/API \
//   cargo check -p svt-av1 --example encode

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let yuv_input = if args.len() == 4 || args.len() == 5 {
        let width: u32 = args[1].parse()?;
        let height: u32 = args[2].parse()?;
        let path = &args[3];
        let ivf_out = if args.len() == 5 {
            Some(args[4].clone())
        } else {
            None
        };
        Some((width, height, path.as_str(), ivf_out))
    } else {
        None
    };

    // Print version from the library (static string)
    let ver = Encoder::version();
    eprintln!("SVT-AV1 version: {}", ver.to_string_lossy());
    Encoder::print_version();

    // Create encoder and default configuration
    let (mut enc, mut cfg): (Encoder, Configuration) = Encoder::init_default()?;

    // Set a few minimal parameters using typed helpers
    if let Some((w, h, _, _)) = yuv_input {
        cfg.set_resolution(w, h);
    } else {
        cfg.set_resolution(320, 240);
    }
    cfg.set_frame_rate(30, 1)
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

    if let Some((width, height, path, ivf_out)) = yuv_input {
        let mut ivf_writer = if let Some(path) = ivf_out {
            let file = File::create(path)?;
            Some(IvfWriter::new(
                BufWriter::new(file),
                width as u16,
                height as u16,
                30,
                1,
            )?)
        } else {
            None
        };

        // Treat the input as a contiguous sequence of raw 8-bit 4:2:0 frames.
        let frame_size = (width as usize * height as usize * 3) / 2;
        let luma_len = (width as usize * height as usize) as usize;
        let chroma_len = luma_len / 4;

        let mut file = File::open(path)?;
        let mut data = vec![0u8; frame_size];
        let mut frame_index: i64 = 0;

        loop {
            match file.read_exact(&mut data) {
                Ok(()) => {
                    let (y_plane, rest) = data.split_at_mut(luma_len);
                    let (u_plane, v_plane) = rest.split_at_mut(chroma_len);

                    let mut io_fmt = sys::enc_bindings::EbSvtIOFormat {
                        luma: y_plane.as_mut_ptr(),
                        cb: u_plane.as_mut_ptr(),
                        cr: v_plane.as_mut_ptr(),
                        y_stride: width,
                        cr_stride: width / 2,
                        cb_stride: width / 2,
                        width,
                        height,
                        org_x: 0,
                        org_y: 0,
                        color_fmt: sys::enc_bindings::EbColorFormat_EB_YUV420,
                        bit_depth: sys::enc_bindings::EbBitDepth_EB_EIGHT_BIT,
                    };

                    let mut pic: BufferHeader = unsafe { std::mem::zeroed() };
                    pic.size = std::mem::size_of::<BufferHeader>() as u32;
                    pic.p_buffer =
                        (&mut io_fmt as *mut sys::enc_bindings::EbSvtIOFormat) as *mut u8;
                    pic.n_filled_len = frame_size as u32;
                    pic.n_alloc_len = frame_size as u32;
                    pic.pts = frame_index;
                    pic.pic_type = sys::enc_bindings::EbAv1PictureType_EB_AV1_INVALID_PICTURE;

                    enc.send_picture(&mut pic)?;
                    frame_index += 1;
                }
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    // Stop cleanly if we hit EOF between frames.
                    break;
                }
                Err(e) => return Err(e.into()),
            }
        }

        // Send an explicit EOS buffer so the encoder knows no more input is coming.
        let mut eos: BufferHeader = unsafe { std::mem::zeroed() };
        eos.size = std::mem::size_of::<BufferHeader>() as u32;
        eos.flags = sys::enc_bindings::EB_BUFFERFLAG_EOS;
        eos.pic_type = sys::enc_bindings::EbAv1PictureType_EB_AV1_INVALID_PICTURE;
        enc.send_picture(&mut eos)?;

        let pic_send_done = true;

        enc.drain_packets(pic_send_done, |pkt| {
            let size = pkt.n_filled_len;
            let pts = pkt.pts;
            eprintln!("got packet: {} bytes, pts={}", size, pts);

            if let Some(writer) = ivf_writer.as_mut() {
                // SAFETY: encoder returns valid buffer with length `n_filled_len`.
                let data = unsafe { std::slice::from_raw_parts(pkt.p_buffer, size as usize) };
                writer
                    .write_frame(data, pts as u64)
                    .expect("write IVF frame");
            }
        })?;
        return Ok(());
    }

    // Retrieve stream header (codec config) if needed
    let mut header_pkt: *mut BufferHeader = std::ptr::null_mut();
    enc.get_stream_header(&mut header_pkt)?;
    unsafe {
        if !header_pkt.is_null() {
            // Process stream header here (contained in header_pkt)
            // Release when done
            enc.stream_header_release(header_pkt)?;
        }
    }

    let pic_send_done = false;

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
        eprintln!(
            "iter packet: {} bytes, flags={}",
            hdr.n_filled_len, hdr.flags
        );
        // Dropping `pkt` releases it back to SVT-AV1
    }

    Ok(())
}
