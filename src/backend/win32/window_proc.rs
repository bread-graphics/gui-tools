// MIT/Apache2 License

use crate::{
    mutex::ShimRwLock,
    runtime::{Runtime, RuntimeInternal},
};
use core::mem;
use winapi::{
    shared::{
        basetsd::LONG_PTR,
        minwindef::{FALSE, LPARAM, LRESULT, UINT, WPARAM},
        windef::HWND,
    },
    um::{errhandlingapi, winuser::*},
};

/// The window procedure for gui-tools objects.
pub unsafe extern "system" fn window_procedure(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // if hwnd is not a window, return to its default window procedure
    if IsWindow(hwnd) == FALSE {
        log::warn!(
            "Window procedure was called with a handle that isn't a window. Deferring to default."
        );
        return DefWindowProcA(hwnd, msg, wparam, lparam);
    }

    // if this is WM_NCCREATE, the window is being created for the first time. this means that the lparam
    // should contain a pointer to our runtime. We can set that to the GWLP_USERDATA variable
    if msg == WM_NCCREATE {
        let create_struct_ptr = mem::transmute::<LPARAM, LPCREATESTRUCTA>(lparam);
        // the lpCreateParams should be a pointer
        let window_object_ptr = (*create_struct_ptr).lpCreateParams;

        // if it is null, return false for error
        // note: just roll with the punch on release mode
        #[cfg(debug_assertions)]
        {
            if window_object_ptr.is_null() {
                log::error!(
                    "Pointer to additional parameter is null. This is probably an internal error."
                );
                return FALSE as LRESULT;
            }
        }

        SetWindowLongPtrA(
            hwnd,
            GWLP_USERDATA,
            window_object_ptr as *const () as LONG_PTR,
        );
        return DefWindowProcA(hwnd, msg, wparam, lparam);
    }

    // now we're guaranteed to exist after creation. get the pointer to the runtime.
    let runtime = GetWindowLongPtrA(hwnd, GWLP_USERDATA);

    // if the pointer is null, this probably isn't our window, just return
    if runtime == 0 {
        // if there is an error, suppress it
        let err = errhandlingapi::GetLastError();
        if err == 1812 {
            // this seems to happen no matter what
            // TODO: prevent this from happening
            log::warn!("GetWindowLongPtr threw error 1812. This is expected.");
            errhandlingapi::SetLastError(0);
        } else if err != 0 {
            log::error!(
                "GetWindowLongPtrA threw error {}. This is ignored, deferring to default window procedure.",
                err
            );
            errhandlingapi::SetLastError(0);
        } else {
            // this is likely an internal error
            log::error!(
                "GetWindowLongPtrA returned a null pointer. This is likely an internal error."
            );
        }

        return DefWindowProcA(hwnd, msg, wparam, lparam);
    }

    // transmute the win32 pointer to the Arc pointer we encoded into it
    let runtime = mem::transmute::<LONG_PTR, *const ShimRwLock<RuntimeInternal>>(runtime);
    let runtime = Runtime::from_ptr(runtime);
    let wruntime = runtime.as_win32().unwrap();

    // get the surface at the location indicated by the pointer
    let surface = if hwnd.is_null() {
        None
    } else {
        runtime.surface_at(hwnd as *const () as usize)
    };

    // translate the message to a gui-tools event
    match super::win32_translate_event(&runtime, surface.as_deref(), msg, wparam, lparam) {
        Ok(events) => wruntime.store_events(events),
        Err(e) => wruntime.store_error(e),
    }

    // forget the runtime so the pointer is still valid
    mem::forget((wruntime, surface));
    mem::forget(runtime);

    DefWindowProcA(hwnd, msg, wparam, lparam)
}
