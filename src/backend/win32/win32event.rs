// MIT/Apache2 License

use crate::{event::Event, runtime::Runtime, surface::Surface};
use storagevec::StorageVec;
use winapi::shared::minwindef::{LPARAM, UINT, WPARAM};

pub fn win32_translate_event(
    runtime: &Runtime,
    surface: Option<&Surface>,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> crate::Result<StorageVec<Event, 5>> {
    unimplemented!()
}
