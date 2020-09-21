// MIT/Apache2 License

use crate::runtime::Runtime;
use core::ptr::NonNull;
use spinny::RwLock;
use x11nas::xlib::Display;

#[cfg(feature = "alloc")]
use hashbrown::HashMap;

#[cfg(not(feature = "alloc"))]
use conquer_once::spin::OnceCell;
#[cfg(not(feature = "alloc"))]
use core::sync::atomic::{AtomicPtr, Ordering};

#[cfg(not(feature = "alloc"))]
static CURRENT_DPY: OnceCell<AtomicPtr<Display>> = OnceCell::uninit();

#[cfg(feature = "alloc")]
static DPY_MAP: RwLock<Option<HashMap<usize, Runtime>>> = RwLock::new(None);

#[cfg(not(feature = "alloc"))]
pub fn get_runtime(dpy: NonNull<Display>) -> Option<Runtime> {
    let current_dpy = CURRENT_DPY.get();
    if current_dpy.is_none() {
        None
    } else if current_dpy.unwrap().load(Ordering::Relaxed) == dpy.as_ptr() {
        Some(unsafe { Runtime::global() })
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
    CURRENT_DPY.init_once(move || AtomicPtr::new(dpy.as_ptr()));
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
