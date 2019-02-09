use std::fs::File;
use std::io::{Error, ErrorKind};
use std::marker::PhantomData;
use std::os::unix::io::{RawFd, AsRawFd};

pub use self::ffi::is_drm_available;

mod ffi;

pub struct Master {}

pub struct Unprivileged {}

pub trait DeviceType {}

impl DeviceType for Master {}
impl DeviceType for Unprivileged {}

pub struct Device<'a, T: DeviceType> {
    file: &'a File,
    phantom: PhantomData<T>,
}

impl<'a> Device<'a, Unprivileged> {
    pub fn new(file: &'a File) -> Self {
        Device::<Unprivileged>{ file: file, phantom: PhantomData }
    }
    
    fn _set_master(&'a self) -> Result<Device<'a, Master>, Error> {
        let fd = self.file.as_raw_fd();
        let master = ffi::set_master(fd);

        if master >= 0 {
            Ok(Device::<Master> { file: self.file, phantom: PhantomData })
        } else {
            Err(Error::new(ErrorKind::PermissionDenied, "Could not become DRM master"))
        }
    }

    pub fn as_master<F>(&'a self, f: F) -> Result<(), Error>
    where F: FnOnce(&Device<Master>) -> Result<(), Error> {
        let master = self._set_master()?;
        f(&master)?;
        master.drop_master();
        Ok(())
    }
}

impl<'a> Device<'a, Master> {

    pub fn drop_master(&'a self) -> Device<'a, Unprivileged> {
        let fd = self.file.as_raw_fd();
        ffi::drop_master(fd);
        Device::<Unprivileged>{ file: &self.file, phantom: PhantomData }
    }

    pub fn get_resources(&'a self) -> Result<ffi::DRMModeRes, Error> {
        let fd = self.file.as_raw_fd();
        ffi::DRMModeRes::new(fd)
    }

    pub fn get_connector(&'a self, connector_id: u32) -> Result<ffi::DRMModeConnector, Error> {
        let fd = self.file.as_raw_fd();
        ffi::DRMModeConnector::new(fd, connector_id)
    }

    #[deprecated]
    pub fn raw_fd(&'a self) -> RawFd {
        self.file.as_raw_fd()
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
