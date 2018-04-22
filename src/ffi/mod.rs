//!
//! Foreign function interface
//!

#![allow(dead_code)]
#![allow(missing_docs)]

use nix::libc::{c_int, c_char};
use nix::Error;
pub use drm_sys::*;

use generic_array::*;
use generic_array::typenum::*;

use std::mem;
use std::cmp;
use std::borrow::Borrow;
use std::os::unix::io::AsRawFd;

pub mod ioctl;

/// Simple trait that gives access to the raw field this struct wraps around.
trait AsRaw<Raw>  {
    fn raw_ref(&self) -> &Raw;
    fn raw_mut(&mut self) -> &mut Raw;
}

// Implement AsRef for AsRaw.
impl<T, Raw> AsRef<Raw> for T where T: AsRaw<Raw> {
    fn as_ref(&self) -> &Raw {
        self.raw_ref()
    }
}

// Implement AsMut for AsRaw.
impl<T, Raw> AsMut<Raw> for T where T: AsRaw<Raw> {
    fn as_mut(&mut self) -> &mut Raw {
        self.raw_mut()
    }
}

/// Many DRM structures have fields that act as pointers to buffers. In libdrm,
/// these buffers are allocated at runtime using `drmMalloc` after determining
/// the size of the buffer.
///
/// However, these buffers tend to be extremely tiny in nature. Therefore, we
/// wrap the DRM structures in a new type that also owns these buffers as
/// fixed-sized arrays. This provides us with two benefits:
///
/// 1. We only need to make the ioctl call once.
/// 2. We do not need to allocate memory on the heap.
///
/// If the actual number of elements exceeds our fixed-length array though, then
/// we will only fill the number of elements we can contain. If this happens on
/// a particular system, it's recommended to increase the length of these buffers
/// and consider filing a bug report.
#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct Buffer<T: Copy, N>(GenericArray<T, N>) where N: ArrayLength<T>;

/// Helper methods that allow us to attach and coerce buffer pointers and sizes.
impl<T, N> Buffer<T, N> where N: ArrayLength<T> {
    /// Attach a buffer to a raw pointer and length.
    fn attach<U>(&self, cnt: &mut u32, ptr: &mut *mut U) {
        *ptr = self.0.as_mut_ptr() as _;
        *cnt = self.0.len() as _;
    }

    /// Coerce a buffer to the maximum size if it's too large.
    fn coerce(&self, cnt: &mut u32) {
        let min = cmp::min(*cnt, self.0.as_slice().len() as _);
        *cnt = min
    }
}

/// 8-element buffer
pub(crate) type Buf8<T> = Buffer<T, U8>;

/// 16-element buffer
pub(crate) type Buf16<T> = Buffer<T, U16>;

/// 32-element buffer
pub(crate) type Buf32<T> = Buffer<T, U32>;

/// Trait for attaching and coercing buffers with underlying FFI structs.
trait BufferSetup {
    fn attach_buffers(&mut self);
    fn coerce_buffers(&mut self);
}

/// Each DRM wrapper we create has an associated FFI command (usually an ioctl)
/// that can be executed.
trait Command {
    /// Very light wrapper for the `ioctl` this type uses.
    fn cmd<B, D>(&mut self, device: &B) -> Result<(), Error>
    where B: Borrow<D>,
          D: AsRawFd;
}

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
struct BusID {
    raw: drm_unique,
    pub unique: Buf32<c_char>
}

impl AsRaw<drm_unique> for BusID {
    fn raw_ref(&self) -> &drm_unique {
        &self.raw
    }

    fn raw_mut(&mut self) -> &mut drm_unique {
        &mut self.raw
    }
}

impl Command for BusID {
    fn cmd<B, D>(&mut self, device: &B) -> Result<(), Error>
    where B: Borrow<D>,
          D: AsRawFd
    {
        unsafe {
            ioctl::get_bus_id(device.borrow().as_raw_fd(), self.as_mut())?
        }
    }
}

impl BufferSetup for BusID {
    fn attach_buffers(&mut self) {
        let mut ptr = &mut self.raw_mut().unique;
        let mut cnt = &mut self.raw_mut().unique_len;
        self.unique.attach(ptr, cnt);
    }

    fn coerce_buffers(&mut self) {
        let mut cnt = &mut self.raw_mut().unique_len;
        self.unique.coerce(cnt);
    }
}

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
struct Client {
    raw: drm_client
}

impl AsRaw<drm_client> for Client {
    fn raw_ref(&self) -> &drm_client {
        &self.raw
    }

    fn raw_mut(&mut self) -> &mut drm_client {
        &mut self.raw
    }
}

impl Command for Client {
    fn cmd<B, D>(&mut self, device: &B) -> Result<(), Error>
    where B: Borrow<D>,
          D: AsRawFd
    {
        unsafe {
            ioctl::get_client(device.borrow().as_raw_fd(), self.as_mut())?
        }
    }
}

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
struct Stats {
    raw: drm_stats
}

impl AsRaw<drm_stats> for Stats {
    fn raw_ref(&self) -> &drm_stats {
        &self.raw
    }

    fn raw_mut(&mut self) -> &mut drm_stats {
        &mut self.raw
    }
}

impl Command for Stats {
    fn cmd<B, D>(&mut self, device: &B) -> Result<(), Error>
    where B: Borrow<D>,
          D: AsRawFd
    {
        unsafe {
            ioctl::get_stats(device.borrow().as_raw_fd(), self.as_mut())?
        }
    }
}

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
struct GetCap {
    raw: drm_get_cap
}

impl AsRaw<drm_get_cap> for GetCap {
    fn raw_ref(&self) -> &drm_get_cap {
        &self.raw
    }

    fn raw_mut(&mut self) -> &mut drm_get_cap {
        &mut self.raw
    }
}

impl Command for GetCap {
    fn cmd<B, D>(&mut self, device: &B) -> Result<(), Error>
    where B: Borrow<D>,
          D: AsRawFd
    {
        unsafe {
            ioctl::get_cap(device.borrow().as_raw_fd(), self.as_mut())?
        }
    }
}

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct SetCap {
    raw: drm_set_client_cap
}

impl AsRaw<drm_set_client_cap> for SetCap {
    fn raw_ref(&self) -> &drm_set_client_cap {
        &self.raw
    }

    fn raw_mut(&mut self) -> &mut drm_set_client_cap {
        &mut self.raw
    }
}

impl Command for SetCap {
    fn cmd<B, D>(&mut self, device: &B) -> Result<(), Error>
    where B: Borrow<D>,
          D: AsRawFd
    {
        unsafe {
            ioctl::set_cap(device.borrow().as_raw_fd(), self.as_mut())?
        }
    }
}

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
struct SetVersion {
    raw: drm_set_version
}

impl AsRaw<drm_set_version> for SetVersion {
    fn raw_ref(&self) -> &drm_set_version {
        &self.raw
    }

    fn raw_mut(&mut self) -> &mut drm_set_version {
        &mut self.raw
    }
}

impl Command for SetVersion {
    fn cmd<B, D>(&mut self, device: &B) -> Result<(), Error>
    where B: Borrow<D>,
          D: AsRawFd
    {
        unsafe {
            ioctl::set_version(device.borrow().as_raw_fd(), self.as_mut())?
        }
    }
}

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
struct GetVersion {
    raw: drm_version,
    name: Buf32<c_char>,
    date: Buf32<c_char>,
    desc: Buf32<c_char>
}

impl AsRaw<drm_version> for GetVersion {
    fn raw_ref(&self) -> &drm_version {
        &self.raw
    }

    fn raw_mut(&mut self) -> &mut drm_set_version {
        &mut self.raw
    }
}

impl Command for GetVersion {
    fn cmd<B, D>(&mut self, device: &B) -> Result<(), Error>
    where B: Borrow<D>,
          D: AsRawFd
    {
        unsafe {
            ioctl::get_version(device.borrow().as_raw_fd(), self.as_mut())?
        }
    }
}

impl BufferSetup for GetVersion {
    fn attach_buffers(&mut self) {
        let mut ptr = &mut self.raw_mut().name;
        let mut cnt = &mut self.raw_mut().name_len;
        self.name.attach(ptr, cnt);

        let mut ptr = &mut self.raw_mut().date;
        let mut cnt = &mut self.raw_mut().date_len;
        self.date.attach(ptr, cnt);

        let mut ptr = &mut self.raw_mut().desc;
        let mut cnt = &mut self.raw_mut().desc_len;
        self.desc.attach(ptr, cnt);
    }

    fn coerce_buffers(&mut self) {
        let mut cnt = &mut self.raw_mut().name_len;
        self.name.coerce(cnt);

        let mut cnt = &mut self.raw_mut().date_len;
        self.date.coerce(cnt);

        let mut cnt = &mut self.raw_mut().desc_len;
        self.desc.coerce(cnt);
    }
}

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct GetToken {
    raw: drm_auth
}

impl AsRaw<drm_auth> for GetToken {
    fn raw_ref(&self) -> &drm_auth {
        &self.raw
    }

    fn raw_mut(&mut self) -> &mut drm_auth {
        &mut self.raw
    }
}

impl Command for GetToken {
    fn cmd<B, D>(&mut self, device: &B) -> Result<(), Error>
    where B: Borrow<D>,
          D: AsRawFd
    {
        unsafe {
            ioctl::get_token(device.borrow().as_raw_fd(), self.as_mut())?
        }
    }
}

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct AuthToken {
    raw: drm_auth
}

impl AsRaw<drm_auth> for AuthToken {
    fn raw_ref(&self) -> &drm_auth {
        &self.raw
    }

    fn raw_mut(&mut self) -> &mut drm_auth {
        &mut self.raw
    }
}

impl Command for AuthToken {
    fn cmd<B, D>(&mut self, device: &B) -> Result<(), Error>
    where B: Borrow<D>,
          D: AsRawFd
    {
        unsafe {
            ioctl::auth_token(device.borrow().as_raw_fd(), self.as_mut())?
        }
    }
}

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
struct IRQControl {
    raw: drm_control
}

impl AsRaw<drm_control> for IRQControl {
    fn raw_ref(&self) -> &drm_control {
        &self.raw
    }

    fn raw_mut(&mut self) -> &mut drm_control {
        &mut self.raw
    }
}

impl Command for IRQControl {
    fn cmd<B, D>(&mut self, device: &B) -> Result<(), Error>
    where B: Borrow<D>,
          D: AsRawFd
    {
        unsafe {
            ioctl::irq_control(device.borrow().as_raw_fd(), self.as_mut())?
        }
    }
}

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
struct WaitVBlank {
    raw: drm_wait_vblank
}

impl AsRaw<drm_wait_vblank> for WaitVBlank {
    fn raw_ref(&self) -> &drm_wait_vblank {
        &self.raw
    }

    fn raw_mut(&mut self) -> &mut drm_wait_vblank {
        &mut self.raw
    }
}

impl Command for WaitVBlank {
    fn cmd<B, D>(&mut self, device: &B) -> Result<(), Error>
    where B: Borrow<D>,
          D: AsRawFd
    {
        unsafe {
            ioctl::wait_vblank(device.borrow().as_raw_fd(), self.as_mut())?
        }
    }
}

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
struct ModesetCtl {
    raw: drm_modeset_ctl
}

impl AsRaw<drm_modeset_ctl> for ModesetCtl {
    fn raw_ref(&self) -> &drm_modeset_ctl {
        &self.raw
    }

    fn raw_mut(&mut self) -> &mut drm_modeset_ctl {
        &mut self.raw
    }
}

impl Command for ModesetCtl {
    fn cmd<B, D>(&mut self, device: &B) -> Result<(), Error>
    where B: Borrow<D>,
          D: AsRawFd
    {
        unsafe {
            ioctl::modeset_ctl(device.borrow().as_raw_fd(), self.as_mut())?
        }
    }
}

pub(crate) mod mode {
    use nix::libc::{c_int, c_uint, uint32_t, uint64_t};
    use nix::Error;
    use super::*;

    pub(crate) type RawHandle = uint32_t;

    #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
    pub(crate) struct CardRes {
        raw: drm_mode_card_res,
        connectors: Buf32<RawHandle>,
        encoders: Buf32<RawHandle>,
        crtcs: Buf32<RawHandle>,
        framebuffers: Buf32<RawHandle>
    }

    impl AsRaw<drm_mode_card_res> for CardRes {
        fn raw_ref(&self) -> &drm_mode_card_res {
            &self.raw
        }

        fn raw_mut(&mut self) -> &mut drm_mode_card_res {
            &mut self.raw
        }
    }

    impl Command for CardRes {
        fn cmd<B, D>(&mut self, device: &B) -> Result<(), Error>
        where B: Borrow<D>,
              D: AsRawFd
        {
            unsafe {
                ioctl::mode::get_resources(device.borrow().as_raw_fd(), self.as_mut())?
            }
        }
    }

    impl BufferSetup for CardRes {
        fn attach_buffers(&mut self) {
            let mut ptr = &mut self.raw_mut().connector_id_ptr;
            let mut cnt = &mut self.raw_mut().count_connectors;
            self.connectors.attach(ptr, cnt);

            let mut ptr = &mut self.raw_mut().encoder_id_ptr;
            let mut cnt = &mut self.raw_mut().count_encoders;
            self.encoders.attach(ptr, cnt);

            let mut ptr = &mut self.raw_mut().crtc_id_ptr;
            let mut cnt = &mut self.raw_mut().count_crtcs;
            self.crtcs.attach(ptr, cnt);

            let mut ptr = &mut self.raw_mut().fb_id_ptr;
            let mut cnt = &mut self.raw_mut().count_fbs;
            self.framebuffers.attach(ptr, cnt);
        }

        fn coerce_buffers(&mut self) {
            let mut cnt = &mut self.raw_mut().count_connectors;
            self.connectors.coerce(cnt);

            let mut cnt = &mut self.raw_mut().count_encoders;
            self.encoders.coerce(cnt);

            let mut cnt = &mut self.raw_mut().count_crtcs;
            self.crtcs.coerce(cnt);

            let mut cnt = &mut self.raw_mut().count_fbs;
            self.framebuffers.coerce(cnt);
        }
    }

    #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
    pub(crate) struct PlaneRes {
        raw: drm_mode_get_plane_res,
        planes: Buf32<RawHandle>
    }

    impl AsRaw<drm_mode_get_plane_res> for PlaneRes {
        fn raw_ref(&self) -> &drm_mode_get_plane_res {
            &self.raw
        }

        fn raw_mut(&mut self) -> &mut drm_mode_get_plane_res {
            &mut self.raw
        }
    }

    impl Command for PlaneRes {
        fn cmd<B, D>(&mut self, device: &B) -> Result<(), Error>
        where B: Borrow<D>,
              D: AsRawFd
        {
            unsafe {
                ioctl::mode::get_plane_resources(device.borrow().as_raw_fd(), self.as_mut())?
            }
        }
    }

    impl BufferSetup for PlaneRes {
        fn attach_buffers(&mut self) {
            let mut ptr = &mut self.raw_mut().plane_id_ptr;
            let mut cnt = &mut self.raw_mut().count_planes;
            self.planes.attach(ptr, cnt);
        }

        fn coerce_buffers(&mut self) {
            let mut cnt = &mut self.raw_mut().count_planes;
            self.planes.coerce(cnt);
        }
    }

    #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
    pub(crate) struct GetConnector {
        raw: drm_mode_get_connector,
        encoders: Buf32<RawHandle>,
        properties: Buf32<RawHandle>,
        prop_values: Buf32<uint64_t>,
        modes: Buf32<drm_mode_modeinfo>
    }

    impl AsRaw<drm_mode_get_connector> for GetConnector {
        fn raw_ref(&self) -> &drm_mode_get_connector {
            &self.raw
        }

        fn raw_mut(&mut self) -> &mut drm_mode_get_connector {
            &mut self.raw
        }
    }

    impl Command for GetConnector {
        fn cmd<B, D>(&mut self, device: &B) -> Result<(), Error>
        where B: Borrow<D>,
              D: AsRawFd
        {
            unsafe {
                ioctl::mode::get_connector(device.borrow().as_raw_fd(), self.as_mut())?
            }
        }
    }

    impl BufferSetup for GetConnector {
        fn attach_buffers(&mut self) {
            let mut ptr = &mut self.raw_mut().encoders_ptr;
            let mut cnt = &mut self.raw_mut().count_encoders;
            self.encoders.attach(ptr, cnt);

            let mut ptr = &mut self.raw_mut().props_ptr;
            let mut cnt = &mut self.raw_mut().count_props;
            self.properties.attach(ptr, cnt);

            let mut ptr = &mut self.raw_mut().prop_values_ptr;
            let mut cnt = &mut self.raw_mut().count_props;
            self.prop_values.attach(ptr, cnt);

            let mut ptr = &mut self.raw_mut().modes_ptr;
            let mut cnt = &mut self.raw_mut().count_modes;
            self.modes.attach(ptr, cnt);
        }

        fn coerce_buffers(&mut self) {
            let mut cnt = &mut self.raw_mut().count_encoders;
            self.encoders.coerce(cnt);

            let mut cnt = &mut self.raw_mut().count_props;
            self.properties.coerce(cnt);

            let mut cnt = &mut self.raw_mut().count_modes;
            self.modes.coerce(cnt);
        }
    }

    #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
    pub(crate) struct GetEncoder {
        raw: drm_mode_get_encoder
    }

    impl AsRaw<drm_mode_get_encoder> for GetEncoder {
        fn raw_ref(&self) -> &drm_mode_get_encoder {
            &self.raw
        }

        fn raw_mut(&mut self) -> &mut drm_mode_get_encoder {
            &mut self.raw
        }
    }

    impl Command for GetEncoder {
        fn cmd<B, D>(&mut self, device: &B) -> Result<(), Error>
        where B: Borrow<D>,
              D: AsRawFd
        {
            unsafe {
                ioctl::mode::get_encoder(device.borrow().as_raw_fd(), self.as_mut())?
            }
        }
    }

    #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
    pub(crate) struct GetCrtc {
        raw: drm_mode_crtc,
        connectors: Buf32<RawHandle>
    }

    impl AsRaw<drm_mode_crtc> for GetCrtc {
        fn raw_ref(&self) -> &drm_mode_crtc {
            &self.raw
        }

        fn raw_mut(&mut self) -> &mut drm_mode_crtc {
            &mut self.raw
        }
    }

    impl Command for GetCrtc {
        fn cmd<B, D>(&mut self, device: &B) -> Result<(), Error>
        where B: Borrow<D>,
              D: AsRawFd
        {
            unsafe {
                ioctl::mode::get_crtc(device.borrow().as_raw_fd(), self.as_mut())?
            }
        }
    }

    impl BufferSetup for GetCrtc {
        fn attach_buffers(&mut self) {
            let mut ptr = &mut self.raw_mut().set_connectors_ptr;
            let mut cnt = &mut self.raw_mut().count_connectors;
            self.connectors.attach(ptr, cnt);
        }

        fn coerce_buffers(&mut self) {
            let mut cnt = &mut self.raw_mut().count_connectors;
            self.connectors.coerce(cnt);
        }
    }

    #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
    pub(crate) struct SetCrtc {
        raw: drm_mode_crtc,
        connectors: Buf32<RawHandle>
    }

    impl AsRaw<drm_mode_crtc> for SetCrtc {
        fn raw_ref(&self) -> &drm_mode_crtc {
            &self.raw
        }

        fn raw_mut(&mut self) -> &mut drm_mode_crtc {
            &mut self.raw
        }
    }

    impl Command for SetCrtc {
        fn cmd<B, D>(&mut self, device: &B) -> Result<(), Error>
        where B: Borrow<D>,
              D: AsRawFd
        {
            unsafe {
                ioctl::mode::set_crtc(device.borrow().as_raw_fd(), self.as_mut())?
            }
        }
    }

    impl BufferSetup for SetCrtc {
        fn attach_buffers(&mut self) {
            let mut ptr = &mut self.raw_mut().set_connectors_ptr;
            let mut cnt = &mut self.raw_mut().count_connectors;
            self.connectors.attach(ptr, cnt);
        }

        fn coerce_buffers(&mut self) {
            let mut cnt = &mut self.raw_mut().count_connectors;
            self.connectors.coerce(cnt);
        }
    }

    #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
    pub(crate) struct GetFB {
        raw: drm_mode_fb_cmd
    }

    impl AsRaw<drm_mode_fb_cmd> for GetFB {
        fn raw_ref(&self) -> &drm_mode_fb_cmd {
            &self.raw
        }

        fn raw_mut(&mut self) -> &mut drm_mode_fb_cmd {
            &mut self.raw
        }
    }

    impl Command for GetFB {
        fn cmd<B, D>(&mut self, device: &B) -> Result<(), Error>
        where B: Borrow<D>,
              D: AsRawFd
        {
            unsafe {
                ioctl::mode::get_fb(device.borrow().as_raw_fd(), self.as_mut())?
            }
        }
    }

    #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
    pub(crate) struct AddFB {
        raw: drm_mode_fb_cmd
    }

    impl AsRaw<drm_mode_fb_cmd> for AddFB {
        fn raw_ref(&self) -> &drm_mode_fb_cmd {
            &self.raw
        }

        fn raw_mut(&mut self) -> &mut drm_mode_fb_cmd {
            &mut self.raw
        }
    }

    impl Command for AddFB {
        fn cmd<B, D>(&mut self, device: &B) -> Result<(), Error>
        where B: Borrow<D>,
              D: AsRawFd
        {
            unsafe {
                ioctl::mode::add_fb(device.borrow().as_raw_fd(), self.as_mut())?
            }
        }
    }


    #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
    pub(crate) struct AddFB2 {
        raw: drm_mode_fb_cmd2
    }

    impl AsRaw<drm_mode_fb_cmd2> for AddFB2 {
        fn raw_ref(&self) -> &drm_mode_fb_cmd2 {
            &self.raw
        }

        fn raw_mut(&mut self) -> &mut drm_mode_fb_cmd2 {
            &mut self.raw
        }
    }

    impl Command for AddFB2 {
        fn cmd<B, D>(&mut self, device: &B) -> Result<(), Error>
        where B: Borrow<D>,
              D: AsRawFd
        {
            unsafe {
                ioctl::mode::add_fb2(device.borrow().as_raw_fd(), self.as_mut())?
            }
        }
    }

    #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
    pub(crate) struct RmFB {
        raw: RawHandle
    }

    impl AsRaw<RawHandle> for RmFB {
        fn raw_ref(&self) -> &RawHandle {
            &self.raw
        }

        fn raw_mut(&mut self) -> &mut RawHandle {
            &mut self.raw
        }
    }

    impl Command for RmFB {
        fn cmd<B, D>(&mut self, device: &B) -> Result<(), Error>
        where B: Borrow<D>,
              D: AsRawFd
        {
            unsafe {
                ioctl::mode::rm_fb(device.borrow().as_raw_fd(), self.as_mut())?
            }
        }
    }

    #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
    pub(crate) struct GetPlane {
        raw: drm_mode_get_plane,
        formats: Buf32<uint32_t>
    }

    impl AsRaw<drm_mode_get_plane> for GetPlane {
        fn raw_ref(&self) -> &drm_mode_get_plane {
            &self.raw
        }

        fn raw_mut(&mut self) -> &mut drm_mode_get_plane {
            &mut self.raw
        }
    }

    impl Command for GetPlane {
        fn cmd<B, D>(&mut self, device: &B) -> Result<(), Error>
        where B: Borrow<D>,
              D: AsRawFd
        {
            unsafe {
                ioctl::mode::get_plane(device.borrow().as_raw_fd(), self.as_mut())?
            }
        }
    }

    impl BufferSetup for GetPlane {
        fn attach_buffers(&mut self) {
            let mut ptr = &mut self.raw_mut().format_type_ptr;
            let mut cnt = &mut self.raw_mut().count_format_types;
            self.formats.attach(ptr, cnt);
        }

        fn coerce_buffers(&mut self) {
            let mut cnt = &mut self.raw_mut().count_format_types;
            self.formats.coerce(cnt);
        }
    }

    #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
    pub(crate) struct SetPlane {
        raw: drm_mode_set_plane
    }

    impl AsRaw<drm_mode_set_plane> for SetPlane {
        fn raw_ref(&self) -> &drm_mode_set_plane {
            &self.raw
        }

        fn raw_mut(&mut self) -> &mut drm_mode_set_plane {
            &mut self.raw
        }
    }

    impl Command for SetPlane {
        fn cmd<B, D>(&mut self, device: &B) -> Result<(), Error>
        where B: Borrow<D>,
              D: AsRawFd
        {
            unsafe {
                ioctl::mode::set_plane(device.borrow().as_raw_fd(), self.as_mut())?
            }
        }
    }

    #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
    pub(crate) struct CreateDumb {
        raw: drm_mode_create_dumb
    }

    impl AsRaw<drm_mode_create_dumb> for CreateDumb {
        fn raw_ref(&self) -> &drm_mode_create_dumb {
            &self.raw
        }

        fn raw_mut(&mut self) -> &mut drm_mode_create_dumb {
            &mut self.raw
        }
    }

    impl Command for CreateDumb {
        fn cmd<B, D>(&mut self, device: &B) -> Result<(), Error>
        where B: Borrow<D>,
              D: AsRawFd
        {
            unsafe {
                ioctl::mode::create_dumb(device.borrow().as_raw_fd(), self.as_mut())?
            }
        }
    }

    #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
    pub(crate) struct MapDumb {
        raw: drm_mode_map_dumb
    }

    impl AsRaw<drm_mode_map_dumb> for MapDumb {
        fn raw_ref(&self) -> &drm_mode_map_dumb {
            &self.raw
        }

        fn raw_mut(&mut self) -> &mut drm_mode_map_dumb {
            &mut self.raw
        }
    }

    impl Command for MapDumb {
        fn cmd<B, D>(&mut self, device: &B) -> Result<(), Error>
        where B: Borrow<D>,
              D: AsRawFd
        {
            unsafe {
                ioctl::mode::map_dumb(device.borrow().as_raw_fd(), self.as_mut())?
            }
        }
    }

    #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
    pub(crate) struct DestroyDumb {
        raw: drm_mode_destroy_dumb
    }

    impl AsRaw<drm_mode_destroy_dumb> for DestroyDumb {
        fn raw_ref(&self) -> &drm_mode_destroy_dumb {
            &self.raw
        }

        fn raw_mut(&mut self) -> &mut drm_mode_destroy_dumb {
            &mut self.raw
        }
    }

    impl Command for DestroyDumb {
        fn cmd<B, D>(&mut self, device: &B) -> Result<(), Error>
        where B: Borrow<D>,
              D: AsRawFd
        {
            unsafe {
                ioctl::mode::destroy_dumb(device.borrow().as_raw_fd(), self.as_mut())?
            }
        }
    }

    #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
    pub(crate) struct Cursor(drm_mode_cursor);

    impl AsRaw<drm_mode_cursor> for Cursor {
        fn raw_ref(&self) -> &drm_mode_cursor {
            &self.raw
        }

        fn raw_mut(&mut self) -> &mut drm_mode_cursor {
            &mut self.raw
        }
    }

    impl Command for Cursor {
        fn cmd<B, D>(&mut self, device: &B) -> Result<(), Error>
        where B: Borrow<D>,
              D: AsRawFd
        {
            unsafe {
                ioctl::mode::cursor(device.borrow().as_raw_fd(), self.as_mut())?
            }
        }
    }

    #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
    pub(crate) struct Cursor2(drm_mode_cursor2);

    impl AsRaw<drm_mode_cursor2> for Cursor2 {
        fn raw_ref(&self) -> &drm_mode_cursor2 {
            &self.raw
        }

        fn raw_mut(&mut self) -> &mut drm_mode_cursor2 {
            &mut self.raw
        }
    }

    impl Command for Cursor2 {
        fn cmd<B, D>(&mut self, device: &B) -> Result<(), Error>
        where B: Borrow<D>,
              D: AsRawFd
        {
            unsafe {
                ioctl::mode::cursor2(device.borrow().as_raw_fd(), self.as_mut())?
            }
        }
    }

    // TODO: Requires some extra work for setting up buffers
    pub(crate) struct GetProperty {
        raw: drm_mode_get_property,
    }

    /*
    wrapper! {
        #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
        struct ConnectorSetProperty(drm_mode_connector_set_property);
        fn ioctl::mode::connector_set_property;
    }

    // TODO: Requires some extra work for setting up buffers
    pub(crate) struct ObjGetProperties {
        raw: drm_mode_obj_get_properties,
        pub prop_buf: Buffer<uint32_t; 32>,
        pub vals_buf: Buffer<uint64_t; 32>
    }

    wrapper! {
        #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
        struct ObjSetProperty(drm_mode_obj_set_property);
        fn ioctl::mode::obj_set_property;
    }
/*
    wrapper! {
        struct CreateBlob(drm_mode_create_blob);
        fn ioctl::mode::create_blob;
    }
    */

    wrapper! {
        #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
        struct DestroyBlob(drm_mode_destroy_blob);
        fn ioctl::mode::destroy_blob;
    }

    wrapper! {
        #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
        struct CrtcPageFlip(drm_mode_crtc_page_flip);
        fn ioctl::mode::crtc_page_flip;
    }

    wrapper! {
        #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
        struct FBDirtyCmd(drm_mode_fb_dirty_cmd);
        fn ioctl::mode::dirty_fb;
    }

    wrapper! {
        #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
        struct Atomic {
            raw: drm_mode_atomic,
            objects: [uint32_t; 32] = [raw.objs_ptr; raw.count_objs],
            count_properties: [uint32_t; 32] = [raw.count_props_ptr; raw.count_objs],
            properties: [uint32_t; 32] = [raw.props_ptr; raw.count_objs],
            prop_values: [uint64_t; 32] = [raw.prop_values_ptr; raw.count_objs]
        }

        fn ioctl::mode::atomic;
    }*/
}

pub(crate) mod gem {
    use nix::libc::c_int;
    use nix::Error;
    use super::*;

    // Underlying type for a GEM handle.
    pub(crate) type RawHandle = u32;

    #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
    pub(crate) struct Open {
        raw: drm_gem_open
    }

    impl AsRaw<drm_gem_open> for Open {
        fn raw_ref(&self) -> &drm_gem_open {
            &self.raw
        }

        fn raw_mut(&mut self) -> &mut drm_gem_open {
            &mut self.raw
        }
    }

    impl Command for Open {
        fn cmd<B, D>(&mut self, device: &B) -> Result<(), Error>
        where B: Borrow<D>,
              D: AsRawFd
        {
            unsafe {
                ioctl::gem::open(device.borrow().as_raw_fd(), self.as_mut())?
            }
        }
    }

    #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
    pub(crate) struct Close {
        raw: drm_gem_close
    }

    impl AsRaw<drm_gem_close> for Close {
        fn raw_ref(&self) -> &drm_gem_close {
            &self.raw
        }

        fn raw_mut(&mut self) -> &mut drm_gem_close {
            &mut self.raw
        }
    }

    impl Command for Close {
        fn cmd<B, D>(&mut self, device: &B) -> Result<(), Error>
        where B: Borrow<D>,
              D: AsRawFd
        {
            unsafe {
                ioctl::gem::close(device.borrow().as_raw_fd(), self.as_mut())?
            }
        }
    }

    #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
    pub(crate) struct Flink {
        raw: drm_gem_flink
    }

    impl AsRaw<drm_gem_flink> for Flink {
        fn raw_ref(&self) -> &drm_gem_flink {
            &self.raw
        }

        fn raw_mut(&mut self) -> &mut drm_gem_flink {
            &mut self.raw
        }
    }

    impl Command for Flink {
        fn cmd<B, D>(&mut self, device: &B) -> Result<(), Error>
        where B: Borrow<D>,
              D: AsRawFd
        {
            unsafe {
                ioctl::gem::flink(device.borrow().as_raw_fd(), self.as_mut())?
            }
        }
    }

    #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
    pub(crate) struct PrimeHandleToFD {
        raw: drm_prime_handle
    }

    impl AsRaw<drm_prime_handle> for PrimeHandleToFD {
        fn raw_ref(&self) -> &drm_prime_handle {
            &self.raw
        }

        fn raw_mut(&mut self) -> &mut drm_prime_handle {
            &mut self.raw
        }
    }

    impl Command for PrimeHandleToFD {
        fn cmd<B, D>(&mut self, device: &B) -> Result<(), Error>
        where B: Borrow<D>,
              D: AsRawFd
        {
            unsafe {
                ioctl::gem::prime_handle_to_fd(device.borrow().as_raw_fd(), self.as_mut())?
            }
        }
    }

    #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq)]
    pub(crate) struct PrimeFDToHandle {
        raw: drm_prime_handle
    }

    impl AsRaw<drm_prime_handle> for PrimeFDToHandle {
        fn raw_ref(&self) -> &drm_prime_handle {
            &self.raw
        }

        fn raw_mut(&mut self) -> &mut drm_prime_handle {
            &mut self.raw
        }
    }

    impl Command for PrimeFDToHandle {
        fn cmd<B, D>(&mut self, device: &B) -> Result<(), Error>
        where B: Borrow<D>,
              D: AsRawFd
        {
            unsafe {
                ioctl::gem::prime_fd_to_handle(device.borrow().as_raw_fd(), self.as_mut())?
            }
        }
    }
}
