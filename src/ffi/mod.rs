#![allow(dead_code)]
extern crate libc;

use libc::{c_int, c_char, uint16_t, uint32_t, uint64_t, c_void};
use std::io::{Error, ErrorKind};
use std::os::unix::io::RawFd;
use std::slice;

#[derive(Debug)]
#[repr(C)]
struct _drmModeRes {
    count_fbs: c_int,
    fbs: *mut uint32_t,

    count_crtcs: c_int,
    crtcs: *mut uint32_t,

    count_connectors: c_int,
    connectors: *mut uint32_t,

    count_encoders: c_int,
    encoders: *mut uint32_t,

    min_width: uint32_t,
    max_width: uint32_t,

    min_height: uint32_t,
    max_height: uint32_t,
}

type DRMModeResPtr = *const _drmModeRes;

const DRM_DISPLAY_MODE_LEN: usize = 32;

#[repr(C)]
struct _drmModeModeInfo {
    clock: uint32_t,
    hdisplay: uint16_t,
    hsync_start: uint16_t,
    hsync_end: uint16_t,
    htotal: uint16_t,
    hskew: uint16_t,
    vdisplay: uint16_t,
    vsync_start: uint16_t,
    vsync_end: uint16_t,
    vtotal: uint16_t,
    vscan: uint16_t,

    vrefresh: uint32_t,

    flags: uint32_t,
    #[link_name = "type"]
    type_: uint32_t,

    name: [c_char; DRM_DISPLAY_MODE_LEN],
}

type DRMModeModeInfoPtr = *const _drmModeModeInfo;

#[derive(Debug)]
#[repr(C)]
enum DRMConnectorType {
    Unknown,
    VGA,
    DVII,
    DVID,
    DVIA,
    Composite,
    SVIDEO,
    LVDS,
    Component,
    NinePinDIN,
    DisplayPort,
    HDMIA,
    HDMIB,
    TV,
    EDP,
    VIRTUAL,
    DSI,
    DPI
}

#[derive(Debug)]
#[repr(C)]
enum DRMModeConnection {
    Connected = 1,
    Disconnected = 2,
    UnknownConnection = 3
}

#[derive(Debug)]
#[repr(C)]
enum DRMModeSubPixel {
    Unknown = 1,
    HorizontalRGB = 2,
    HorizontalBGR = 3,
    VerticalRGB = 4,
    VerticalBGR = 5,
    None = 6
}

#[derive(Debug)]
#[repr(C)]
struct _drmModeConnector {
    connector_id: uint32_t,
    encoder_id: uint32_t,
    connector_type: DRMConnectorType,
    connector_type_id: uint32_t,
    connection: DRMModeConnection,
    #[link_name = "mmWidth"]
    mm_width: uint32_t,
    #[link_name = "mmHeight"]
    mm_height: uint32_t,
    subpixel: DRMModeSubPixel,

    count_modes: c_int,
    modes: DRMModeModeInfoPtr,

    count_props: c_int,
    props: *mut uint32_t,
    prop_values: *mut uint64_t,

    count_encoders: c_int,
    encoders: *mut uint32_t,
}

type DRMModeConnectorPtr = *const _drmModeConnector;

#[link(name="drm")]
extern "C" {
    fn drmAvailable() -> c_int;
    fn drmSetMaster(fd: c_int) -> c_int;
    fn drmDropMaster(fd: c_int) -> c_int;
    fn drmModeGetResources(fd: c_int) -> DRMModeResPtr;
    fn drmModeFreeResources(ptr: DRMModeResPtr) -> c_void;
    fn drmModeGetConnector(fd: c_int, connectorId: uint32_t) -> DRMModeConnectorPtr;
    fn drmModeFreeConnector(ptr: DRMModeConnectorPtr) -> c_void;
    fn drmModeFreeModeInfo(ptr: DRMModeModeInfoPtr) -> c_void;
}

pub fn is_drm_available() -> bool {
    unsafe { drmAvailable() != 0 }
}

pub fn set_master(fd: RawFd) -> c_int {
    unsafe { drmSetMaster(fd) }
}

pub fn drop_master(fd: RawFd) -> c_int {
    unsafe { drmDropMaster(fd) }
}

#[derive(Debug)]
pub struct DRMModeRes {
    raw: *const _drmModeRes,
    pub framebuffers: Vec<u32>,
    pub crtcs: Vec<u32>,
    pub connectors: Vec<u32>,
    pub encoders: Vec<u32>,
    pub min_width: u32,
    pub max_width: u32,
    pub min_height: u32,
    pub max_height: u32
}

impl DRMModeRes {
    pub fn new(fd: RawFd) -> Result<Self, Error> {
        let opt_r = unsafe { drmModeGetResources(fd).as_ref() };

        match opt_r {
            Some(r) => {
                let fb_count = r.count_fbs as usize;
                let crtc_count = r.count_crtcs as usize;
                let conn_count = r.count_connectors as usize;
                let enc_count = r.count_encoders as usize;
                let res = unsafe {
                    DRMModeRes {
                        raw: r,
                        framebuffers: Vec::from_raw_parts(r.fbs, fb_count, fb_count),
                        crtcs: Vec::from_raw_parts(r.crtcs, crtc_count, crtc_count),
                        connectors: Vec::from_raw_parts(r.connectors, conn_count, conn_count),
                        encoders: Vec::from_raw_parts(r.encoders, enc_count, enc_count),
                        min_width: r.min_width,
                        max_width: r.max_width,
                        min_height: r.min_height,
                        max_height: r.max_height,
                    }
                };
                Ok(res)
            },
            None =>
                Err(Error::new(ErrorKind::Other, "Could not get DRM mode resources"))
        }
    }
}

impl Drop for DRMModeRes {
    fn drop(&mut self) {
        unsafe { drmModeFreeResources(self.raw) };
    }
}

#[derive(Debug)]
struct DRMModeConnectorModes {
    count: usize,
    modes: DRMModeModeInfoPtr,
}

impl std::ops::Deref for DRMModeConnectorModes {
    type Target = [_drmModeModeInfo];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.modes, self.count) }
    }
}

impl Drop for DRMModeConnectorModes {
    fn drop(&mut self) {
        unsafe { drmModeFreeModeInfo(self.modes) };
    }
}

#[derive(Debug)]
struct DRMModeConnectorProps {
    count: usize,
    props: *mut u32,
}

impl std::ops::Deref for DRMModeConnectorProps {
    type Target = [u32];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.props, self.count) }
    }
}

impl Drop for DRMModeConnectorProps {
    fn drop(&mut self) {
        unsafe { libc::free(self.props as *mut c_void) };
    }
}

#[derive(Debug)]
struct DRMModeConnectorPropValues {
    count: usize,
    prop_values: *mut u64,
}

impl std::ops::Deref for DRMModeConnectorPropValues {
    type Target = [u64];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.prop_values, self.count) }
    }
}

impl Drop for DRMModeConnectorPropValues {
    fn drop(&mut self) {
        unsafe { libc::free(self.prop_values as *mut c_void) };
    }
}

#[derive(Debug)]
struct DRMModeConnectorEncoders {
    count: usize,
    encoders: *mut u32,
}

impl std::ops::Deref for DRMModeConnectorEncoders {
    type Target = [u32];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.encoders, self.count) }
    }
}

impl Drop for DRMModeConnectorEncoders {
    fn drop(&mut self) {
        unsafe { libc::free(self.encoders as *mut c_void) };
    }
}

#[derive(Debug)]
pub struct DRMModeConnector<'a> {
    raw: *const _drmModeConnector,
    connector_id: u32,
    encoder_id: u32,
    connector_type: &'a DRMConnectorType,
    connector_type_id: u32,
    connection: &'a DRMModeConnection,

    mm_width: u32,
    mm_height: u32,
    subpixel: &'a DRMModeSubPixel,

    modes: DRMModeConnectorModes,

    props: DRMModeConnectorProps,
    prop_values: DRMModeConnectorPropValues,

    encoders: DRMModeConnectorEncoders,
}

impl<'a> DRMModeConnector<'a> {
    pub fn new(fd: RawFd, connector_id: u32) -> Result<Self, Error> {
        let opt_c = unsafe { drmModeGetConnector(fd, connector_id).as_ref() };
        match opt_c {
            Some(raw) => {
                let c =
                    DRMModeConnector {
                        raw: raw,
                        connector_id: raw.connector_id,
                        encoder_id: raw.encoder_id,
                        connector_type: &raw.connector_type,
                        connector_type_id: raw.connector_type_id,
                        connection: &raw.connection,
                        mm_width: raw.mm_width,
                        mm_height: raw.mm_height,
                        subpixel: &raw.subpixel,
                        modes: DRMModeConnectorModes { count: raw.count_modes as usize, modes: raw.modes },
                        props: DRMModeConnectorProps { count: raw.count_props as usize, props: raw.props },
                        prop_values: DRMModeConnectorPropValues { count: raw.count_props as usize, prop_values: raw.prop_values },
                        encoders: DRMModeConnectorEncoders { count: raw.count_encoders as usize, encoders: raw.encoders }
                    };
                Ok(c)
            },
            None =>
                Err(Error::new(ErrorKind::Other, "Unable to get mode connector"))
        }
    }
}

impl<'a> Drop for DRMModeConnector<'a> {
    fn drop(&mut self) {
        unsafe { drmModeFreeConnector(self.raw) };
    }
}
