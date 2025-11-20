#[cfg(not(feature = "decoder"))]
fn main() {
    eprintln!(
        "decoder example requires: cargo run -p svt-av1 --features decoder --example decode -- <file.ivf>"
    );
}

#[cfg(feature = "decoder")]
mod decode_example {
    use std::error::Error as StdError;
    use std::fs;
    use svt_av1::decoder::{BufferHeader, Decoder, FrameInfo, StreamInfo};
    use svt_av1::Error;
    use svt_av1_sys as sys;

    #[derive(Debug)]
    struct IvfHeader {
        width: u16,
        height: u16,
        frame_rate: u32,
        time_scale: u32,
        frame_count: u32,
    }

    fn parse_ivf(data: &[u8]) -> Result<(IvfHeader, Vec<&[u8]>), Box<dyn StdError>> {
        if data.len() < 32 {
            return Err("IVF: too short".into());
        }
        if &data[0..4] != b"DKIF" {
            return Err("IVF: missing DKIF header".into());
        }
        let fourcc = &data[8..12];
        if fourcc != b"AV01" {
            return Err("IVF: unexpected fourcc".into());
        }
        let width = u16::from_le_bytes([data[12], data[13]]);
        let height = u16::from_le_bytes([data[14], data[15]]);
        let frame_rate = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);
        let time_scale = u32::from_le_bytes([data[20], data[21], data[22], data[23]]);
        let frame_count = u32::from_le_bytes([data[24], data[25], data[26], data[27]]);

        let mut offset = 32usize;
        let mut frames = Vec::new();
        while offset + 12 <= data.len() {
            let size = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]) as usize;
            // timestamp currently ignored
            offset += 12;
            if offset + size > data.len() {
                break;
            }
            frames.push(&data[offset..offset + size]);
            offset += size;
        }

        Ok((
            IvfHeader {
                width,
                height,
                frame_rate,
                time_scale,
                frame_count,
            },
            frames,
        ))
    }

    pub fn run() -> Result<(), Box<dyn StdError>> {
        let args: Vec<String> = std::env::args().collect();
        if args.len() != 2 {
            eprintln!("usage: decode <input.ivf>");
            std::process::exit(1);
        }
        let data = fs::read(&args[1])?;
        let (hdr, frames) = parse_ivf(&data)?;
        eprintln!(
            "IVF input: {}x{} fps={}/{} frames={}",
            hdr.width, hdr.height, hdr.frame_rate, hdr.time_scale, hdr.frame_count
        );

        let (mut dec, mut cfg) = Decoder::init_default()?;
        cfg.max_picture_width = hdr.width as u32;
        cfg.max_picture_height = hdr.height as u32;
        cfg.max_bit_depth = sys::dec_bindings::EbBitDepth_EB_EIGHT_BIT;
        cfg.max_color_format = sys::dec_bindings::EbColorFormat_EB_YUV420;
        cfg.frames_to_be_decoded = hdr.frame_count as u64;
        cfg.eight_bit_output = 1;
        cfg.is_16bit_pipeline = 0;
        dec.set_parameter(&cfg)?;
        dec.init()?;

        let eb_no_error_empty_queue = sys::dec_bindings::EbErrorType_EB_NoErrorEmptyQueue as i32;
        let mut stream_info: StreamInfo = unsafe { std::mem::zeroed() };
        let mut frame_info: FrameInfo = unsafe { std::mem::zeroed() };
        let mut decoded = 0u32;

        for frame in frames {
            dec.send_packet(frame)?;
            loop {
                let mut pic: BufferHeader = unsafe { std::mem::zeroed() };
                match dec.get_picture(&mut pic, &mut stream_info, &mut frame_info) {
                    Ok(()) => {
                        eprintln!(
                            "decoded frame {} pts={} dims={}x{} flags={}",
                            decoded,
                            pic.pts,
                            stream_info.max_picture_width,
                            stream_info.max_picture_height,
                            pic.flags
                        );
                        decoded += 1;
                    }
                    Err(Error::Code(c)) if c == eb_no_error_empty_queue => break,
                    Err(e) => return Err(e.into()),
                }
            }
        }

        println!(
            "Decoded {} frames; stream {}x{}",
            decoded, hdr.width, hdr.height
        );
        Ok(())
    }
}

#[cfg(feature = "decoder")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    decode_example::run()
}
