#![doc = include_str!("../README.md")]
#![deny(unsafe_op_in_unsafe_fn)]
use svt_av1_sys as sys;

use thiserror::Error;

/// Errors surfaced by the safe wrapper.
///
/// Most variants map directly to SVT-AV1 `EbErrorType` codes returned from FFI
/// entry points; `Null` guards against accidentally passing null handles.
#[derive(Debug, Error)]
pub enum Error {
    #[error("SVT-AV1 error code {0}")]
    Code(i32),
    #[error("Null pointer")]
    Null,
}

/// Result alias using the wrapper [`Error`] type.
pub type Result<T> = std::result::Result<T, Error>;

/// Converts an SVT-AV1 status code to a [`Result`].
fn ok(code: i32) -> Result<()> {
    if code == 0 {
        Ok(())
    } else {
        Err(Error::Code(code))
    }
}

/// Strongly-typed helpers and enums for configuring the encoder.
///
/// These map directly to fields on `EbSvtAv1EncConfiguration` so callers can
/// configure the encoder with type safety.
pub mod config {
    use super::sys;

    /// Bit depth of the input pixels.
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    #[repr(u32)]
    pub enum BitDepth {
        Eight = 8,
        Ten = 10,
        Twelve = 12,
    }

    /// Chroma subsampling mode.
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    #[repr(u32)]
    pub enum ColorFormat {
        Yuv400 = 0,
        Yuv420 = 1,
        Yuv422 = 2,
        Yuv444 = 3,
    }

    /// Luma range interpretation.
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    #[repr(u32)]
    pub enum ColorRange {
        Studio = 0,
        Full = 1,
    }

    /// Chroma sample location.
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    #[repr(u32)]
    pub enum ChromaSamplePosition {
        Unknown = 0,
        Vertical = 1,
        Colocated = 2,
    }

    /// AV1 profile selector.
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    #[repr(u32)]
    pub enum Profile {
        Main = 0,
        High = 1,
        Professional = 2,
    }

    /// AV1 level tier.
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    #[repr(u32)]
    pub enum Tier {
        Main = 0,
        High = 1,
    }

    /// Rate control mode.
    ///
    /// `CqpOrCrf` matches the SVT-AV1 CQP/CRF setting.
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    #[repr(u8)]
    pub enum RcMode {
        CqpOrCrf = 0,
        Vbr = 1,
        Cbr = 2,
    }

    /// Intra refresh strategy.
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    #[repr(u32)]
    pub enum IntraRefreshType {
        FwdKey = 1,
        Key = 2,
    }

    /// Convenience extension methods for `EbSvtAv1EncConfiguration`.
    ///
    /// Each method mutates the configuration in place and returns `self` for
    /// easy chaining when constructing encoder settings.
    pub trait ConfigExt {
        /// Sets the encoded frame resolution in pixels.
        fn set_resolution(&mut self, width: u32, height: u32) -> &mut Self;
        /// Sets the input frame rate as a rational numerator/denominator pair.
        fn set_frame_rate(&mut self, num: u32, den: u32) -> &mut Self;
        /// Selects encoder input bit depth.
        fn set_bit_depth(&mut self, depth: BitDepth) -> &mut Self;
        /// Sets the chroma subsampling format.
        fn set_color_format(&mut self, fmt: ColorFormat) -> &mut Self;
        /// Sets the luma range interpretation.
        fn set_color_range(&mut self, range: ColorRange) -> &mut Self;
        /// Sets the chroma sample position.
        fn set_chroma_sample_position(&mut self, csp: ChromaSamplePosition) -> &mut Self;
        /// Chooses the AV1 profile.
        fn set_profile(&mut self, profile: Profile) -> &mut Self;
        /// Sets the tier for level signalling.
        fn set_tier(&mut self, tier: Tier) -> &mut Self;
        /// Lets the encoder auto-select the level.
        fn set_level_auto(&mut self) -> &mut Self;
        /// Overrides the level using an explicit level code.
        fn set_level_code(&mut self, level_code: u32) -> &mut Self;
        /// Sets the rate control mode.
        fn set_rc_mode(&mut self, mode: RcMode) -> &mut Self;
        /// Sets the target bitrate in bits per second.
        fn set_target_bitrate(&mut self, bps: u32) -> &mut Self;
        /// Sets the fixed quantizer parameter (QP).
        fn set_qp(&mut self, qp: u32) -> &mut Self;
        /// Configures intra refresh behavior.
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
            self.rate_control_mode = mode as u8;
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
            self.enable_roi_map = enable;
            self
        }
        fn enable_recon(&mut self, enable: bool) -> &mut Self {
            self.recon_enabled = enable;
            self
        }
    }
}

#[cfg(feature = "encoder")]
/// Safe encoder wrappers over the SVT-AV1 C API (enabled by the `encoder` feature).
pub mod encoder {
    use super::*;
    use std::ffi::{CStr, CString};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Mutex, OnceLock};

    // SVT-AV1's runtime CPU detection (RTCD) setup in v4.0.1 is guarded by a
    // non-thread-safe static `first_call_setup` boolean inside the C library.
    //
    // If multiple threads call `svt_av1_enc_init` concurrently for the very
    // first encoder initialization, the library can log errors like:
    //   `Pointer "..." is set before!`
    //
    // Serialize the first successful encoder init to avoid races. After that,
    // the library's own guard is set and subsequent inits can proceed without
    // locking.
    static SVT_ENC_RTCD_INIT_DONE: AtomicBool = AtomicBool::new(false);
    static SVT_ENC_RTCD_INIT_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    /// Raw frame/packet header exchanged with the C API.
    pub use sys::enc_bindings::EbBufferHeaderType as BufferHeader;
    /// Raw encoder component handle from SVT-AV1.
    pub use sys::enc_bindings::EbComponentType as Component;
    /// Raw per-picture private data node used to pass ROI maps and other events.
    pub use sys::enc_bindings::EbPrivDataNode as PrivDataNode;
    /// Encoder configuration struct matching the SVT-AV1 C layout.
    pub use sys::enc_bindings::EbSvtAv1EncConfiguration as Configuration;
    /// Raw ROI map types from the C API.
    pub use sys::enc_bindings::SvtAv1RoiMap as RoiMap;
    pub use sys::enc_bindings::SvtAv1RoiMapEvt as RoiMapEvent;
    /// Discriminant used in `PrivDataNode.node_type` for ROI map events.
    pub const ROI_MAP_EVENT: sys::enc_bindings::PrivDataType =
        sys::enc_bindings::PrivDataType_ROI_MAP_EVENT;
    // The public API primarily uses BufferHeader and Configuration for I/O and params.

    /// Owned encoder handle that cleans up via `Drop`.
    pub struct Handle(*mut Component);

    unsafe impl Send for Handle {}
    unsafe impl Sync for Handle {}

    impl Default for Handle {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Handle {
        /// Creates a null handle suitable for initialization calls.
        pub fn new() -> Self {
            Self(std::ptr::null_mut())
        }
        /// Returns a mutable pointer to the inner handle for FFI APIs that fill it.
        pub fn as_mut_ptr(&mut self) -> *mut *mut Component {
            &mut self.0 as *mut _
        }
        /// Returns the raw handle pointer.
        pub fn as_ptr(&self) -> *mut Component {
            self.0
        }
        /// Returns true if the handle has not been initialized.
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

    /// RAII encoder entry point wrapping the SVT-AV1 handle.
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

        /// Creates a new encoder handle and fills a default configuration.
        pub fn init_default() -> Result<(Self, Configuration)> {
            let mut handle = Handle::new();
            let mut cfg: Configuration = unsafe { std::mem::zeroed() };
            let code = unsafe {
                sys::enc_bindings::svt_av1_enc_init_handle(handle.as_mut_ptr(), &mut cfg)
            };
            super::ok(code)?;
            Ok((Self { handle }, cfg))
        }

        /// Applies an `EbSvtAv1EncConfiguration` to an existing handle.
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

        /// Finalizes initialization after parameters have been configured.
        pub fn init(&mut self) -> Result<()> {
            if !SVT_ENC_RTCD_INIT_DONE.load(Ordering::Acquire) {
                let lock = SVT_ENC_RTCD_INIT_LOCK.get_or_init(|| Mutex::new(()));
                let _guard = lock.lock().unwrap_or_else(|e| e.into_inner());
                if !SVT_ENC_RTCD_INIT_DONE.load(Ordering::Acquire) {
                    let code = unsafe { sys::enc_bindings::svt_av1_enc_init(self.handle.as_ptr()) };
                    let res = super::ok(code);
                    if res.is_ok() {
                        SVT_ENC_RTCD_INIT_DONE.store(true, Ordering::Release);
                    }
                    return res;
                }
            }

            let code = unsafe { sys::enc_bindings::svt_av1_enc_init(self.handle.as_ptr()) };
            super::ok(code)
        }

        /// Submits a picture to the encoder. Ownership of `p_buffer` remains with the caller.
        pub fn send_picture(&mut self, pic: &mut BufferHeader) -> Result<()> {
            let code = unsafe {
                sys::enc_bindings::svt_av1_enc_send_picture(self.handle.as_ptr(), pic as *mut _)
            };
            super::ok(code)
        }

        /// Attempts to dequeue the next output packet, returning `None` if the queue is empty.
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

        /// Releases a packet previously obtained by [`get_packet`](Self::get_packet).
        pub fn release_out_buffer(&mut self, packet: &mut *mut BufferHeader) {
            unsafe { sys::enc_bindings::svt_av1_enc_release_out_buffer(packet as *mut _) };
        }

        /// Retrieves the codec stream header packet.
        pub fn get_stream_header(&mut self, packet: &mut *mut BufferHeader) -> Result<()> {
            let code = unsafe {
                sys::enc_bindings::svt_av1_enc_stream_header(self.handle.as_ptr(), packet as *mut _)
            };
            super::ok(code)
        }

        /// Releases a stream header packet returned by [`get_stream_header`](Self::get_stream_header).
        ///
        /// # Safety
        ///
        /// `packet` must be a valid stream header previously returned by
        /// `get_stream_header` for this encoder instance and not already released.
        pub unsafe fn stream_header_release(&mut self, packet: *mut BufferHeader) -> Result<()> {
            let code = unsafe { sys::enc_bindings::svt_av1_enc_stream_header_release(packet) };
            super::ok(code)
        }

        /// Copies reconstructed frame data into the provided buffer header.
        pub fn get_recon(&mut self, buffer: &mut BufferHeader) -> Result<()> {
            let code = unsafe {
                sys::enc_bindings::svt_av1_get_recon(self.handle.as_ptr(), buffer as *mut _)
            };
            super::ok(code)
        }

        /// Queries encoder stream metadata by id into a caller-allocated buffer.
        ///
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
        /// Returns the raw `EbBufferHeaderType` pointer.
        pub fn as_ptr(&self) -> *mut BufferHeader {
            self.0
        }
        /// Returns a shared reference to the buffer header.
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

    /// Iterator over encoder output packets; each yielded packet is freed on drop.
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
/// Safe decoder wrappers over the SVT-AV1 C API (enabled by the `decoder` feature).
pub mod decoder {
    use super::*;

    /// Frame metadata returned alongside decoded buffers.
    pub use sys::dec_bindings::EbAV1FrameInfo as FrameInfo;
    /// Stream metadata returned during decoding.
    pub use sys::dec_bindings::EbAV1StreamInfo as StreamInfo;
    /// Raw buffer header used for decoder output.
    pub use sys::dec_bindings::EbBufferHeaderType as BufferHeader;
    /// Raw decoder component handle from SVT-AV1.
    pub use sys::dec_bindings::EbComponentType as Component;
    /// Decoder configuration struct matching the SVT-AV1 C layout.
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
        /// Creates a null decoder handle suitable for initialization calls.
        pub fn new() -> Self {
            Self(std::ptr::null_mut())
        }
        /// Returns a mutable pointer to the inner handle for FFI APIs that fill it.
        pub fn as_mut_ptr(&mut self) -> *mut *mut Component {
            &mut self.0 as *mut _
        }
        /// Returns the raw handle pointer.
        pub fn as_ptr(&self) -> *mut Component {
            self.0
        }
        /// Returns true if the handle has not been initialized.
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

    /// RAII decoder entry point wrapping the SVT-AV1 handle.
    pub struct Decoder {
        handle: Handle,
    }

    impl Decoder {
        /// Creates a new decoder handle and fills a default configuration.
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

        /// Applies an `EbSvtAv1DecConfiguration` to an existing handle.
        pub fn set_parameter(&mut self, cfg: &Configuration) -> Result<()> {
            let code = unsafe {
                sys::dec_bindings::svt_av1_dec_set_parameter(
                    self.handle.as_ptr(),
                    cfg as *const _ as *mut _,
                )
            };
            super::ok(code)
        }

        /// Finalizes initialization after parameters have been configured.
        pub fn init(&mut self) -> Result<()> {
            let code = unsafe { sys::dec_bindings::svt_av1_dec_init(self.handle.as_ptr()) };
            super::ok(code)
        }

        /// Feeds an encoded AV1 packet into the decoder.
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

        /// Retrieves the next decoded picture and associated metadata.
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
