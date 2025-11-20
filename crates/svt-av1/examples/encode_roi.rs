use svt_av1::config::{BitDepth, ColorFormat, ConfigExt, IntraRefreshType, Profile, RcMode, Tier};
use svt_av1::encoder::{
    BufferHeader, Configuration, Encoder, PrivDataNode, RoiMap, RoiMapEvent, ROI_MAP_EVENT,
};
use svt_av1_sys as sys;

// Minimal example demonstrating how to enable ROI map support
// and construct per-picture ROI metadata structures.
//
// This is a header-only check example (no linking required) and is
// meant to compile in CI with vendored headers:
//   SVT_AV1_NO_PKG_CONFIG=1 SVT_AV1_INCLUDE_DIR=vendor/SVT-AV1/Source/API \
//   cargo check -p svt-av1 --example encode_roi
//
// NOTE: This example does not actually feed frames to the encoder. It
// focuses on showing how to build the ROI structures and the private
// data node that would be attached to an `EbBufferHeaderType`.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_enc, mut cfg): (Encoder, Configuration) = Encoder::init_default()?;

    // Configure encoder and enable ROI maps.
    cfg.set_resolution(320, 240)
        .set_frame_rate(30, 1)
        .set_bit_depth(BitDepth::Eight)
        .set_color_format(ColorFormat::Yuv420)
        .set_profile(Profile::Main)
        .set_tier(Tier::Main)
        .set_level_auto()
        .set_rc_mode(RcMode::Vbr)
        .set_qp(40)
        .set_intra_refresh(IntraRefreshType::FwdKey)
        .enable_roi_map(true);

    // For illustration, build a trivial ROI map event.
    //
    // In a real encoder, you would compute:
    // - b64_seg_map: segment IDs per 64x64 block
    // - seg_qp: per-segment QP deltas
    let mut seg_qp = [0i16; 8];
    seg_qp[0] = -4; // segment 0: slightly higher quality

    let mut dummy_seg_map = [0u8; 1]; // placeholder map

    let mut roi_evt = RoiMapEvent {
        start_picture_number: 0,
        b64_seg_map: dummy_seg_map.as_mut_ptr(),
        seg_qp,
        max_seg_id: 0,
        next: std::ptr::null_mut(),
    };

    let mut roi = RoiMap {
        evt_num: 1,
        evt_list: &mut roi_evt,
        cur_evt: &mut roi_evt,
        qp_map: std::ptr::null_mut(),
        buf: std::ptr::null_mut(),
    };

    let mut node = PrivDataNode {
        node_type: ROI_MAP_EVENT,
        data: &mut roi as *mut _ as *mut std::ffi::c_void,
        size: std::mem::size_of::<RoiMap>() as u32,
        next: std::ptr::null_mut(),
    };

    // Attach the ROI node to an input buffer header before sending it to the encoder.
    let mut pic: BufferHeader = unsafe { std::mem::zeroed() };
    pic.size = std::mem::size_of::<BufferHeader>() as u32;
    pic.pic_type = sys::enc_bindings::EbAv1PictureType_EB_AV1_INVALID_PICTURE;
    pic.p_app_private = &mut node as *mut _ as *mut std::ffi::c_void;

    // In a full encode flow you would then set parameters and send the picture:
    // enc.set_parameter(&cfg)?;
    // enc.init()?;
    // enc.send_picture(&mut pic)?;
    // enc.drain_packets(true, |_| {})?;
    // The SVT-AV1 library deep copies `p_app_private`, so dropping `node` after
    // send_picture is fine.

    println!("ROI example: configuration and ROI structures constructed.");

    Ok(())
}
