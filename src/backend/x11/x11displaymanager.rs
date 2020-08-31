// MIT/Apache2 License

use crate::runtime::Runtime;
use core::ptr::NonNull;
use spinny::RwLock;
use x11nas::xlib::Display;

#[cfg(feature = "alloc")]
use hashbrown::HashMap;

#[cfg(not(feature = "alloc"))]
static CURRENT_DPY: RwLock<Option<NonNull<Display>>> = RwLock::new(None);

#[cfg(feature = "alloc")]
static DPY_MAP: RwLock<Option<HashMap<usize, Runtime>>> = RwLock::new(None);

#[cfg(not(feature = "alloc"))]
pub fn get_runtime(dpy: NonNull<Display>) -> Option<Runtime> {
    if CURRENT_DPY.read() == dpy {
        Some(Runtime::global())
    } else {
        None
    }
}

#[cfg(feature = "alloc")]
pub fn get_runtime(dpy: NonNull<Display>) -> Option<Runtime> {
    let key = dpy.as_ptr() as *const () as usize;

    let dpy_map = DPY_MAP.read();
    if dpy_map.is_none() {
        return None;
    }
    match dpy_map.as_ref().unwrap().get(&key) {
        Some(r) => Some(r.clone()),
        None => None,
    }
}

#[cfg(not(feature = "alloc"))]
pub fn set_runtime(dpy: NonNull<Display>, _runtime: Runtime) {
    *CURRENT_DPR.write() = Some(dpy);
}

#[cfg(feature = "alloc")]
pub fn set_runtime(dpy: NonNull<Display>, runtime: Runtime) {
    let mut dpy_map = DPY_MAP.write();
    if dpy_map.is_none() {
        *dpy_map = Some(HashMap::new());
    }
    dpy_map
        .as_mut()
        .unwrap()
        .insert(dpy.as_ptr() as *const () as usize, runtime);
}
