// MIT/Apache2 License

//! This is a testing backend, in order to test the finer details of the abstractions without actually
//! linking to a native library.

use conquer_once::spin::OnceCell;
use euclid::Size2D;
use gui_tools::{backend::{BackendType, Backend}, geometry::Pixel};
use storagevec::StorageVec;

mod testruntime;
mod testsurface;

pub const TESTING_BACKEND_TY: BackendType = BackendType::OtherStr("Testing Backend");

pub struct VirtualMonitorInformation {
    pub dimensions: Size2D<u32, Pixel>,
}

pub struct VirtualSetupInformation {
    pub monitors: StorageVec<VirtualMonitorInformation, 4>,
}

const SETUP_INFORMATION: OnceCell<VirtualSetupInformation> = OnceCell::uninit();

fn open_function() -> crate::Result<(usize, RuntimeInner)> {

}
