#![deny(unsafe_op_in_unsafe_fn)]
use svt_av1_sys as sys;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("SVT-AV1 error code {0}")]
    Code(i32),
    #[error("Null pointer")]
    Null,
}

pub type Result<T> = std::result::Result<T, Error>;

fn ok(code: i32) -> Result<()> {
    if code == 0 {
        Ok(())
    } else {
        Err(Error::Code(code))
    }
}

/// Strongly-typed helpers and enums for configuring the encoder.
pub mod config {
    use super::sys;

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    #[repr(u32)]
    pub enum BitDepth {
        Eight = 8,
        Ten = 10,
        Twelve = 12,
    }

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    #[repr(u32)]
    pub enum ColorFormat {
        Yuv400 = 0,
        Yuv420 = 1,
        Yuv422 = 2,
        Yuv444 = 3,
    }

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    #[repr(u32)]
    pub enum ColorRange {
        Studio = 0,
        Full = 1,
    }

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    #[repr(u32)]
    pub enum ChromaSamplePosition {
        Unknown = 0,
        Vertical = 1,
        Colocated = 2,
    }

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    #[repr(u32)]
    pub enum Profile {
        Main = 0,
        High = 1,
        Professional = 2,
    }

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    #[repr(u32)]
    pub enum Tier {
        Main = 0,
        High = 1,
    }

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    #[repr(u8)]
    pub enum RcMode {
        CqpOrCrf = 0,
        Vbr = 1,
        Cbr = 2,
    }

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    #[repr(u32)]
    pub enum IntraRefreshType {
        FwdKey = 1,
        Key = 2,
    }

    /// Convenience extension methods for `EbSvtAv1EncConfiguration`.
    pub trait ConfigExt {
        fn set_resolution(&mut self, width: u32, height: u32) -> &mut Self;
        fn set_frame_rate(&mut self, num: u32, den: u32) -> &mut Self;
        fn set_bit_depth(&mut self, depth: BitDepth) -> &mut Self;
        fn set_color_format(&mut self, fmt: ColorFormat) -> &mut Self;
        fn set_color_range(&mut self, range: ColorRange) -> &mut Self;
        fn set_chroma_sample_position(&mut self, csp: ChromaSamplePosition) -> &mut Self;
        fn set_profile(&mut self, profile: Profile) -> &mut Self;
        fn set_tier(&mut self, tier: Tier) -> &mut Self;
        fn set_level_auto(&mut self) -> &mut Self;
        fn set_level_code(&mut self, level_code: u32) -> &mut Self;
        fn set_rc_mode(&mut self, mode: RcMode) -> &mut Self;
        fn set_target_bitrate(&mut self, bps: u32) -> &mut Self;
        fn set_qp(&mut self, qp: u32) -> &mut Self;
        fn set_intra_refresh(&mut self, t: IntraRefreshType) -> &mut Self;
        /// Enable or disable ROI map usage in the encoder configuration.
        ///
        /// When enabled, per-picture ROI maps can be supplied via `EbPrivDataNode`
        /// with `ROI_MAP_EVENT` attached to `BufferHeader.p_app_private`.
        fn enable_roi_map(&mut self, enable: bool) -> &mut Self;
        fn enable_recon(&mut self, enable: bool) -> &mut Self;
    }

    impl ConfigExt for sys::enc_bindings::EbSvtAv1EncConfiguration {
        fn set_resolution(&mut self, width: u32, height: u32) -> &mut Self {
            self.source_width = width;
            self.source_height = height;
            self
        }
        fn set_frame_rate(&mut self, num: u32, den: u32) -> &mut Self {
            self.frame_rate_numerator = num;
            self.frame_rate_denominator = den;
            self
        }
        fn set_bit_depth(&mut self, depth: BitDepth) -> &mut Self {
            self.encoder_bit_depth = depth as u32;
            self
        }
        fn set_color_format(&mut self, fmt: ColorFormat) -> &mut Self {
            self.encoder_color_format = fmt as u32;
            self
        }
        fn set_color_range(&mut self, range: ColorRange) -> &mut Self {
            self.color_range = range as u32;
            self
        }
        fn set_chroma_sample_position(&mut self, csp: ChromaSamplePosition) -> &mut Self {
            self.chroma_sample_position = csp as u32;
            self
        }
        fn set_profile(&mut self, profile: Profile) -> &mut Self {
            self.profile = profile as u32;
            self
        }
        fn set_tier(&mut self, tier: Tier) -> &mut Self {
            self.tier = tier as u32;
            self
        }
        fn set_level_auto(&mut self) -> &mut Self {
            self.level = 0;
            self
        }
        fn set_level_code(&mut self, level_code: u32) -> &mut Self {
            self.level = level_code;
            self
        }
        fn set_rc_mode(&mut self, mode: RcMode) -> &mut Self {
            self.rate_control_mode = mode as u32;
            self
        }
        fn set_target_bitrate(&mut self, bps: u32) -> &mut Self {
            self.target_bit_rate = bps;
            self
        }
        fn set_qp(&mut self, qp: u32) -> &mut Self {
            self.qp = qp;
            self
        }
        fn set_intra_refresh(&mut self, t: IntraRefreshType) -> &mut Self {
            self.intra_refresh_type = t as u32;
            self
        }
        fn enable_roi_map(&mut self, enable: bool) -> &mut Self {
            self.enable_roi_map = enable as u8;
            self
        }
        fn enable_recon(&mut self, enable: bool) -> &mut Self {
            self.recon_enabled = enable as u8;
            self
        }
    }
}

#[cfg(feature = "encoder")]
pub mod encoder {
    use super::*;
    use std::ffi::{CStr, CString};

    pub use sys::enc_bindings::EbBufferHeaderType as BufferHeader;
    pub use sys::enc_bindings::EbComponentType as Component;
    /// Raw per-picture private data node used to pass ROI maps and other events.
    pub use sys::enc_bindings::EbPrivDataNode as PrivDataNode;
    pub use sys::enc_bindings::EbSvtAv1EncConfiguration as Configuration;
    /// Raw ROI map types from the C API.
    pub use sys::enc_bindings::SvtAv1RoiMap as RoiMap;
    pub use sys::enc_bindings::SvtAv1RoiMapEvt as RoiMapEvent;
    /// Discriminant used in `PrivDataNode.node_type` for ROI map events.
    pub const ROI_MAP_EVENT: sys::enc_bindings::PrivDataType =
        sys::enc_bindings::PrivDataType_ROI_MAP_EVENT;
    // The public API primarily uses BufferHeader and Configuration for I/O and params.

    pub struct Handle(*mut Component);

    unsafe impl Send for Handle {}
    unsafe impl Sync for Handle {}

    impl Default for Handle {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Handle {
        pub fn new() -> Self {
            Self(std::ptr::null_mut())
        }
        pub fn as_mut_ptr(&mut self) -> *mut *mut Component {
            &mut self.0 as *mut _
        }
        pub fn as_ptr(&self) -> *mut Component {
            self.0
        }
        pub fn is_null(&self) -> bool {
            self.0.is_null()
        }
    }

    impl Drop for Handle {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe {
                    let _ = sys::enc_bindings::svt_av1_enc_deinit_handle(self.as_ptr());
                }
                self.0 = std::ptr::null_mut();
            }
        }
    }

    pub struct Encoder {
        handle: Handle,
    }

    impl Encoder {
        /// Returns a static version string from the library.
        pub fn version() -> &'static CStr {
            unsafe { CStr::from_ptr(sys::enc_bindings::svt_av1_get_version()) }
        }

        /// Prints version/build info to stderr or SVT_LOG_FILE (if set).
        pub fn print_version() {
            unsafe { sys::enc_bindings::svt_av1_print_version() }
        }

        pub fn init_default() -> Result<(Self, Configuration)> {
            let mut handle = Handle::new();
            let mut cfg: Configuration = unsafe { std::mem::zeroed() };
            let code = unsafe {
                sys::enc_bindings::svt_av1_enc_init_handle(
                    handle.as_mut_ptr(),
                    std::ptr::null_mut(),
                    &mut cfg,
                )
            };
            super::ok(code)?;
            Ok((Self { handle }, cfg))
        }

        pub fn set_parameter(&mut self, cfg: &Configuration) -> Result<()> {
            let code = unsafe {
                sys::enc_bindings::svt_av1_enc_set_parameter(
                    self.handle.as_ptr(),
                    cfg as *const _ as *mut _,
                )
            };
            super::ok(code)
        }

        /// Convenience to set a single parameter by name/value using the C parser.
        pub fn parse_parameter(cfg: &mut Configuration, name: &CStr, value: &CStr) -> Result<()> {
            let code = unsafe {
                sys::enc_bindings::svt_av1_enc_parse_parameter(
                    cfg as *mut _,
                    name.as_ptr(),
                    value.as_ptr(),
                )
            };
            super::ok(code)
        }

        /// Convenience string version of `parse_parameter`. Returns EB_ErrorBadParameter on failure.
        pub fn parse_parameter_str(cfg: &mut Configuration, name: &str, value: &str) -> Result<()> {
            let n = CString::new(name).map_err(|_| Error::Code(0x80001005u32 as i32))?; // EB_ErrorBadParameter
            let v = CString::new(value).map_err(|_| Error::Code(0x80001005u32 as i32))?;
            Self::parse_parameter(cfg, &n, &v)
        }

        pub fn init(&mut self) -> Result<()> {
            let code = unsafe { sys::enc_bindings::svt_av1_enc_init(self.handle.as_ptr()) };
            super::ok(code)
        }

        pub fn send_picture(&mut self, pic: &mut BufferHeader) -> Result<()> {
            let code = unsafe {
                sys::enc_bindings::svt_av1_enc_send_picture(self.handle.as_ptr(), pic as *mut _)
            };
            super::ok(code)
        }

        pub fn get_packet(&mut self, pic_send_done: bool) -> Result<Option<*mut BufferHeader>> {
            // EB_NoErrorEmptyQueue indicates no packet available yet; not an error.
            const EB_NO_ERROR_EMPTY_QUEUE: i32 =
                sys::enc_bindings::EbErrorType_EB_NoErrorEmptyQueue;
            let mut packet: *mut BufferHeader = std::ptr::null_mut();
            let code: i32 = unsafe {
                sys::enc_bindings::svt_av1_enc_get_packet(
                    self.handle.as_ptr(),
                    &mut packet as *mut _,
                    if pic_send_done { 1 } else { 0 },
                )
            };
            if code == 0 {
                return Ok(Some(packet));
            }
            if code == EB_NO_ERROR_EMPTY_QUEUE {
                return Ok(None);
            }
            Err(super::Error::Code(code))
        }

        pub fn release_out_buffer(&mut self, packet: &mut *mut BufferHeader) {
            unsafe { sys::enc_bindings::svt_av1_enc_release_out_buffer(packet as *mut _) };
        }

        pub fn get_stream_header(&mut self, packet: &mut *mut BufferHeader) -> Result<()> {
            let code = unsafe {
                sys::enc_bindings::svt_av1_enc_stream_header(self.handle.as_ptr(), packet as *mut _)
            };
            super::ok(code)
        }

        /// # Safety
        ///
        /// `packet` must be a valid stream header previously returned by
        /// `get_stream_header` for this encoder instance and not already released.
        pub unsafe fn stream_header_release(&mut self, packet: *mut BufferHeader) -> Result<()> {
            let code = unsafe { sys::enc_bindings::svt_av1_enc_stream_header_release(packet) };
            super::ok(code)
        }

        pub fn get_recon(&mut self, buffer: &mut BufferHeader) -> Result<()> {
            let code = unsafe {
                sys::enc_bindings::svt_av1_get_recon(self.handle.as_ptr(), buffer as *mut _)
            };
            super::ok(code)
        }

        /// # Safety
        ///
        /// `info` must point to a writable buffer matching the requested `id`
        /// for this encoder instance, as defined by the SVT-AV1 API.
        pub unsafe fn get_stream_info(
            &mut self,
            id: u32,
            info: *mut std::ffi::c_void,
        ) -> Result<()> {
            let code = unsafe {
                sys::enc_bindings::svt_av1_enc_get_stream_info(self.handle.as_ptr(), id, info)
            };
            super::ok(code)
        }

        /// Drain available packets non-blocking and call `f` for each.
        /// Stops when the queue is empty. Set `pic_send_done` to true after feeding all pictures.
        pub fn drain_packets<F>(&mut self, pic_send_done: bool, mut f: F) -> Result<()>
        where
            F: FnMut(&BufferHeader),
        {
            loop {
                match self.get_packet(pic_send_done)? {
                    Some(ptr) => {
                        // SAFETY: FFI returns valid pointer to BufferHeaderType until released
                        let header = unsafe { &*ptr };
                        f(header);
                        let mut p = ptr;
                        self.release_out_buffer(&mut p);
                    }
                    None => break Ok(()),
                }
            }
        }

        /// Returns an iterator that yields packets (RAII-released on drop of each item) until empty.
        pub fn packets<'a>(&'a mut self, pic_send_done: bool) -> PacketIter<'a> {
            PacketIter {
                enc: self,
                pic_send_done,
            }
        }
    }

    impl Drop for Encoder {
        fn drop(&mut self) {
            // Safe to call; ignore errors in Drop
            let _ = unsafe { sys::enc_bindings::svt_av1_enc_deinit(self.handle.as_ptr()) };
        }
    }

    /// RAII packet wrapper: releases the underlying buffer on drop.
    pub struct Packet(*mut BufferHeader);
    impl Packet {
        pub fn as_ptr(&self) -> *mut BufferHeader {
            self.0
        }
        pub fn header(&self) -> &BufferHeader {
            unsafe { &*self.0 }
        }
    }
    impl Drop for Packet {
        fn drop(&mut self) {
            if !self.0.is_null() {
                let mut p = self.0;
                unsafe { sys::enc_bindings::svt_av1_enc_release_out_buffer(&mut p as *mut _) };
                self.0 = std::ptr::null_mut();
            }
        }
    }

    pub struct PacketIter<'a> {
        enc: &'a mut Encoder,
        pic_send_done: bool,
    }
    impl<'a> Iterator for PacketIter<'a> {
        type Item = Result<Packet>;
        fn next(&mut self) -> Option<Self::Item> {
            match self.enc.get_packet(self.pic_send_done) {
                Ok(Some(ptr)) => Some(Ok(Packet(ptr))),
                Ok(None) => None,
                Err(e) => Some(Err(e)),
            }
        }
    }
}

#[cfg(feature = "decoder")]
pub mod decoder {
    use super::*;

    pub use sys::dec_bindings::EbAV1FrameInfo as FrameInfo;
    pub use sys::dec_bindings::EbAV1StreamInfo as StreamInfo;
    pub use sys::dec_bindings::EbBufferHeaderType as BufferHeader;
    pub use sys::dec_bindings::EbComponentType as Component;
    pub use sys::dec_bindings::EbSvtAv1DecConfiguration as Configuration;

    pub struct Handle(*mut Component);
    unsafe impl Send for Handle {}
    unsafe impl Sync for Handle {}

    impl Default for Handle {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Handle {
        pub fn new() -> Self {
            Self(std::ptr::null_mut())
        }
        pub fn as_mut_ptr(&mut self) -> *mut *mut Component {
            &mut self.0 as *mut _
        }
        pub fn as_ptr(&self) -> *mut Component {
            self.0
        }
        pub fn is_null(&self) -> bool {
            self.0.is_null()
        }
    }

    impl Drop for Handle {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe {
                    sys::dec_bindings::svt_av1_dec_deinit_handle(self.as_ptr());
                }
                self.0 = std::ptr::null_mut();
            }
        }
    }

    pub struct Decoder {
        handle: Handle,
    }

    impl Decoder {
        pub fn init_default() -> Result<(Self, Configuration)> {
            let mut handle = Handle::new();
            let mut cfg: Configuration = unsafe { std::mem::zeroed() };
            let code = unsafe {
                sys::dec_bindings::svt_av1_dec_init_handle(
                    handle.as_mut_ptr(),
                    std::ptr::null_mut(),
                    &mut cfg,
                )
            };
            super::ok(code)?;
            Ok((Self { handle }, cfg))
        }

        pub fn set_parameter(&mut self, cfg: &Configuration) -> Result<()> {
            let code = unsafe {
                sys::dec_bindings::svt_av1_dec_set_parameter(
                    self.handle.as_ptr(),
                    cfg as *const _ as *mut _,
                )
            };
            super::ok(code)
        }

        pub fn init(&mut self) -> Result<()> {
            let code = unsafe { sys::dec_bindings::svt_av1_dec_init(self.handle.as_ptr()) };
            super::ok(code)
        }

        pub fn send_packet(&mut self, data: &[u8]) -> Result<()> {
            let code = unsafe {
                sys::dec_bindings::svt_av1_dec_frame(
                    self.handle.as_ptr(),
                    data.as_ptr(),
                    data.len(),
                    0,
                )
            };
            super::ok(code)
        }

        pub fn get_picture(
            &mut self,
            picture: &mut BufferHeader,
            stream_info: &mut StreamInfo,
            frame_info: &mut FrameInfo,
        ) -> Result<()> {
            let code = unsafe {
                sys::dec_bindings::svt_av1_dec_get_picture(
                    self.handle.as_ptr(),
                    picture as *mut _,
                    stream_info as *mut _,
                    frame_info as *mut _,
                )
            };
            super::ok(code)
        }
    }

    impl Drop for Decoder {
        fn drop(&mut self) {
            // svt_av1_dec_deinit caused double free or crash, relying on Handle::drop
        }
    }
}

#[cfg(test)]
mod tests;
/*
fn probe_roi() {
    let buf = sys::enc_bindings::EbBufferHeaderType {
        roi_map: (),
        ..Default::default()
    };
}
 */
