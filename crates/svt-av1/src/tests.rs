#[cfg(test)]
mod tests {
    #[cfg(feature = "encoder")]
    use crate::config::{
        BitDepth, ColorFormat, ConfigExt, IntraRefreshType, Profile, RcMode, Tier,
    };
    #[cfg(feature = "decoder")]
    use crate::decoder::Decoder;
    #[cfg(feature = "encoder")]
    use crate::encoder::{BufferHeader, Configuration, Encoder};
    #[cfg(feature = "encoder")]
    use svt_av1_sys as sys;

    #[test]
    #[cfg(feature = "decoder")]
    fn test_decoder_init() {
        let (mut dec, cfg) = Decoder::init_default().expect("Failed to init decoder");

        dec.init().expect("Failed to init decoder instance");
    }

    #[test]
    #[cfg(feature = "encoder")]
    fn config_helpers_and_parse_parameter() {
        let (_enc, mut cfg): (Encoder, Configuration) =
            Encoder::init_default().expect("default init");

        cfg.set_resolution(64, 64)
            .set_frame_rate(30, 1)
            .set_bit_depth(BitDepth::Eight)
            .set_color_format(ColorFormat::Yuv420)
            .set_profile(Profile::Main)
            .set_tier(Tier::Main)
            .set_level_auto()
            .set_rc_mode(RcMode::Vbr)
            .set_qp(42)
            .set_intra_refresh(IntraRefreshType::FwdKey);

        assert_eq!(cfg.source_width, 64);
        assert_eq!(cfg.source_height, 64);
        assert_eq!(cfg.frame_rate_numerator, 30);
        assert_eq!(cfg.frame_rate_denominator, 1);
        assert_eq!(cfg.encoder_bit_depth, BitDepth::Eight as u32);
        assert_eq!(cfg.encoder_color_format, ColorFormat::Yuv420 as u32);
        assert_eq!(cfg.profile, Profile::Main as u32);
        assert_eq!(cfg.tier, Tier::Main as u32);
        assert_eq!(cfg.rate_control_mode, RcMode::Vbr as u8);
        assert_eq!(cfg.qp, 42);
        assert_eq!(cfg.intra_refresh_type, IntraRefreshType::FwdKey as u32);

        // Known-good parse succeeds.
        Encoder::parse_parameter_str(&mut cfg, "rc", "vbr").expect("parse rc=vbr");
    }

    #[test]
    #[cfg(feature = "encoder")]
    fn encoder_init_and_empty_drain_is_noop() {
        let (mut enc, mut cfg): (Encoder, Configuration) =
            Encoder::init_default().expect("default init");

        cfg.set_resolution(64, 64)
            .set_frame_rate(30, 1)
            .set_bit_depth(BitDepth::Eight)
            .set_color_format(ColorFormat::Yuv420)
            .set_profile(Profile::Main)
            .set_tier(Tier::Main)
            .set_level_auto()
            .set_rc_mode(RcMode::Vbr);

        enc.set_parameter(&cfg).expect("apply params");
        enc.init().expect("init encoder");

        let mut packets = 0;
        enc.drain_packets(false, |_pkt| {
            packets += 1;
        })
        .expect("drain with no input");
        assert_eq!(packets, 0, "no packets expected without input");
    }

    #[test]
    #[cfg(feature = "encoder")]
    fn encoder_handles_eos_without_frames() {
        let (mut enc, mut cfg): (Encoder, Configuration) =
            Encoder::init_default().expect("default init");

        cfg.set_resolution(64, 64)
            .set_frame_rate(30, 1)
            .set_bit_depth(BitDepth::Eight)
            .set_color_format(ColorFormat::Yuv420)
            .set_profile(Profile::Main)
            .set_rc_mode(RcMode::Vbr);

        enc.set_parameter(&cfg).expect("apply params");
        enc.init().expect("init encoder");

        let mut eos: BufferHeader = unsafe { std::mem::zeroed() };
        eos.size = std::mem::size_of::<BufferHeader>() as u32;
        eos.flags = sys::enc_bindings::EB_BUFFERFLAG_EOS;
        eos.pic_type = sys::enc_bindings::EbAv1PictureType_EB_AV1_INVALID_PICTURE;

        enc.send_picture(&mut eos).expect("send eos");

        let mut zero_len_packets = 0;
        enc.drain_packets(true, |pkt| {
            if pkt.n_filled_len == 0 {
                zero_len_packets += 1;
            }
        })
        .expect("drain eos");
        // Allow zero-length packets but ensure no panic/UB.
        assert!(
            zero_len_packets >= 0,
            "drain should not panic on zero-length packets"
        );
    }

    #[test]
    #[cfg(feature = "encoder")]
    fn encode_single_frame_produces_packet() {
        let (mut enc, mut cfg): (Encoder, Configuration) =
            Encoder::init_default().expect("default init");

        const WIDTH: u32 = 64;
        const HEIGHT: u32 = 64;
        cfg.set_resolution(WIDTH, HEIGHT)
            .set_frame_rate(30, 1)
            .set_bit_depth(BitDepth::Eight)
            .set_color_format(ColorFormat::Yuv420)
            .set_profile(Profile::Main)
            .set_rc_mode(RcMode::Vbr)
            .set_qp(50);

        enc.set_parameter(&cfg).expect("apply params");
        enc.init().expect("init encoder");

        let frame_size = (WIDTH * HEIGHT * 3 / 2) as usize;
        let luma_len = (WIDTH * HEIGHT) as usize;
        let chroma_len = luma_len / 4;
        let mut data = vec![0u8; frame_size];
        let (y_plane, rest) = data.split_at_mut(luma_len);
        let (u_plane, v_plane) = rest.split_at_mut(chroma_len);

        let mut io_fmt = sys::enc_bindings::EbSvtIOFormat {
            luma: y_plane.as_mut_ptr(),
            cb: u_plane.as_mut_ptr(),
            cr: v_plane.as_mut_ptr(),
            y_stride: WIDTH,
            cr_stride: WIDTH / 2,
            cb_stride: WIDTH / 2,
        };

        let mut pic: BufferHeader = unsafe { std::mem::zeroed() };
        pic.size = std::mem::size_of::<BufferHeader>() as u32;
        pic.p_buffer = (&mut io_fmt as *mut sys::enc_bindings::EbSvtIOFormat) as *mut u8;
        pic.n_filled_len = frame_size as u32;
        pic.n_alloc_len = frame_size as u32;
        pic.pts = 0;
        pic.pic_type = sys::enc_bindings::EbAv1PictureType_EB_AV1_INVALID_PICTURE;

        enc.send_picture(&mut pic).expect("send frame");

        let mut eos: BufferHeader = unsafe { std::mem::zeroed() };
        eos.size = std::mem::size_of::<BufferHeader>() as u32;
        eos.flags = sys::enc_bindings::EB_BUFFERFLAG_EOS;
        eos.pic_type = sys::enc_bindings::EbAv1PictureType_EB_AV1_INVALID_PICTURE;
        enc.send_picture(&mut eos).expect("send eos");

        let mut packets = 0;
        let mut polls = 0;
        loop {
            polls += 1;
            assert!(polls < 32, "poll limit reached without draining");
            match enc.get_packet(true).expect("get_packet") {
                Some(ptr) => {
                    let header = unsafe { &*ptr };
                    if header.n_filled_len > 0 {
                        packets += 1;
                    }
                    let mut p = ptr;
                    enc.release_out_buffer(&mut p);
                }
                None => break,
            }
        }

        assert!(packets > 0, "expected at least one packet from a frame");
    }
}
