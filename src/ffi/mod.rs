#![allow(dead_code)]
extern crate libc;

use std::ffi::{CStr, CString};
use libc::{c_int, c_char, c_void};
use std::io::{Error, ErrorKind};
use std::os::unix::io::RawFd;
use std::slice;

#[derive(Debug)]
#[repr(C)]
struct _drmModeRes {
    count_fbs: c_int,
    fbs: *mut u32,

    count_crtcs: c_int,
    crtcs: *mut u32,

    count_connectors: c_int,
    connectors: *mut u32,

    count_encoders: c_int,
    encoders: *mut u32,

    min_width: u32,
    max_width: u32,

    min_height: u32,
    max_height: u32,
}

type DRMModeResPtr = *const _drmModeRes;

const DRM_DISPLAY_MODE_LEN: usize = 32;

#[derive(Debug)]
#[repr(C)]
pub struct _drmModeModeInfo {
    clock: u32,
    hdisplay: u16,
    hsync_start: u16,
    hsync_end: u16,
    htotal: u16,
    hskew: u16,
    vdisplay: u16,
    vsync_start: u16,
    vsync_end: u16,
    vtotal: u16,
    vscan: u16,

    vrefresh: u32,

    flags: u32,
    #[link_name = "type"]
    type_: u32,

    name: [c_char; DRM_DISPLAY_MODE_LEN],
}

type DRMModeModeInfoPtr = *const _drmModeModeInfo;

#[derive(Debug)]
pub struct DRMDimensionInfo {
    display: u16,
    sync_start: u16,
    sync_end: u16,
    total: u16
}

#[derive(Debug)]
pub struct DRMModeModeInfo {
    raw: DRMModeModeInfoPtr,
    clock: u32,
    hinfo: DRMDimensionInfo,
    vinfo: DRMDimensionInfo,
    hskew: u16,
    vscan: u16,
    vrefresh: u32,
    flags: u32,
    type_: u32,
    name: CString
}

impl<'a> DRMModeModeInfo {
    pub fn new(raw: &_drmModeModeInfo) -> Result<Self, Error> {
        let name = unsafe { CStr::from_ptr(raw.name.as_ptr()) };
        Ok(DRMModeModeInfo {
            raw: raw,
            clock: raw.clock,
            hinfo: DRMDimensionInfo { display: raw.hdisplay,
                                      sync_start: raw.hsync_start,
                                      sync_end: raw.hsync_end,
                                      total: raw.htotal
            },
            vinfo: DRMDimensionInfo { display: raw.vdisplay,
                                      sync_start: raw.vsync_start,
                                      sync_end: raw.vsync_end,
                                      total: raw.vtotal
            },
            hskew: raw.hskew,
            vscan: raw.vscan,
            vrefresh: raw.vrefresh,
            flags: raw.flags,
            type_: raw.type_,
            name: name.to_owned()
        })
    }
}

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
    connector_id: u32,
    encoder_id: u32,
    connector_type: DRMConnectorType,
    connector_type_id: u32,
    connection: DRMModeConnection,
    #[link_name = "mmWidth"]
    mm_width: u32,
    #[link_name = "mmHeight"]
    mm_height: u32,
    subpixel: DRMModeSubPixel,

    count_modes: c_int,
    modes: DRMModeModeInfoPtr,

    count_props: c_int,
    props: *mut u32,
    prop_values: *mut u64,

    count_encoders: c_int,
    encoders: *mut u32,
}

type DRMModeConnectorPtr = *const _drmModeConnector;

#[link(name="drm")]
extern "C" {
    fn drmAvailable() -> c_int;
    fn drmSetMaster(fd: c_int) -> c_int;
    fn drmDropMaster(fd: c_int) -> c_int;
    fn drmModeGetResources(fd: c_int) -> DRMModeResPtr;
    fn drmModeFreeResources(ptr: DRMModeResPtr) -> c_void;
    fn drmModeGetConnector(fd: c_int, connectorId: u32) -> DRMModeConnectorPtr;
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
pub struct DRMModeResFramebuffers {
    count: usize,
    fbs: *mut u32,
}

impl std::ops::Deref for DRMModeResFramebuffers {
    type Target = [u32];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.fbs, self.count) }
    }
}

impl Drop for DRMModeResFramebuffers {
    fn drop(&mut self) {
        unsafe { libc::free(self.fbs as *mut c_void) };
    }
}

#[derive(Debug)]
pub struct DRMModeResCrtcs {
    count: usize,
    crtcs: *mut u32,
}

impl std::ops::Deref for DRMModeResCrtcs {
    type Target = [u32];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.crtcs, self.count) }
    }
}

impl Drop for DRMModeResCrtcs {
    fn drop(&mut self) {
        unsafe { libc::free(self.crtcs as *mut c_void) };
    }
}

#[derive(Debug)]
pub struct DRMModeResConnectors {
    count: usize,
    conns: *mut u32,
}

impl std::ops::Deref for DRMModeResConnectors {
    type Target = [u32];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.conns, self.count) }
    }
}

impl Drop for DRMModeResConnectors {
    fn drop(&mut self) {
        unsafe { libc::free(self.conns as *mut c_void) };
    }
}

#[derive(Debug)]
pub struct DRMModeResEncoders {
    count: usize,
    encs: *mut u32,
}

impl std::ops::Deref for DRMModeResEncoders {
    type Target = [u32];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.encs, self.count) }
    }
}

impl Drop for DRMModeResEncoders {
    fn drop(&mut self) {
        unsafe { libc::free(self.encs as *mut c_void) };
    }
}


#[derive(Debug)]
pub struct DRMModeRes {
    raw: *const _drmModeRes,
    pub framebuffers: DRMModeResFramebuffers,
    pub crtcs: DRMModeResCrtcs,
    pub connectors: DRMModeResConnectors,
    pub encoders: DRMModeResEncoders,
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
                let res =
                    DRMModeRes {
                        raw: r,
                        framebuffers: DRMModeResFramebuffers { count: fb_count, fbs: r.fbs },
                        crtcs: DRMModeResCrtcs { count: crtc_count, crtcs: r.crtcs },
                        connectors: DRMModeResConnectors { count: conn_count, conns: r.connectors },
                        encoders: DRMModeResEncoders { count: enc_count, encs: r.encoders },
                        min_width: r.min_width,
                        max_width: r.max_width,
                        min_height: r.min_height,
                        max_height: r.max_height,
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
pub struct DRMModeConnectorModes {
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

    pub modes: Vec<DRMModeModeInfo>,

    props: DRMModeConnectorProps,
    prop_values: DRMModeConnectorPropValues,

    encoders: DRMModeConnectorEncoders,
}

impl<'a> DRMModeConnector<'a> {
    pub fn new(fd: RawFd, connector_id: u32) -> Result<Self, Error> {
        let opt_c = unsafe { drmModeGetConnector(fd, connector_id).as_ref() };
        match opt_c {
            Some(raw) => {
                let raw_modes = DRMModeConnectorModes { count: raw.count_modes as usize, modes: raw.modes };
                let mut modes: Vec<DRMModeModeInfo> = Vec::new();
                for mode in 0..raw_modes.count {
                    let mode_info = DRMModeModeInfo::new(&raw_modes[mode])?;
                    modes.push(mode_info);
                }
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
                        modes: modes,
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
