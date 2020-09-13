// MIT/Apache2 License

use crate::{
    event::{Event, EventType},
    graphics::Graphics,
    keyboard::{KeyInfo, KeyType},
    mouse::MouseButton,
    runtime::Runtime,
    surface::{Surface, SurfaceBackend},
};
use core::{
    mem::{self, MaybeUninit},
    ptr::NonNull,
};
use cty::c_int;
use euclid::{point2, size2};
use storagevec::StorageVec;
use winapi::{
    shared::{
        minwindef::{HIWORD, LOWORD, LPARAM, LRESULT, UINT, WPARAM},
        ntdef::USHORT,
        windef::{HDC, POINT, RECT},
        windowsx::{GET_X_LPARAM, GET_Y_LPARAM},
    },
    um::{wingdi, winuser::*},
};

pub fn win32_translate_event(
    runtime: &Runtime,
    surface: Option<&Surface>,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> crate::Result<(StorageVec<Event, 5>, Option<LRESULT>)> {
    macro_rules! unwrap_surface {
        ($surface: expr, $call: ident) => {{
            match ($surface) {
                Some(surface) => surface,
                None => {
                    log::warn!(
                        "Called {} message without initialized window",
                        stringify!($call)
                    );
                    return Ok((StorageVec::new(), None));
                }
            }
        }};
    }

    let mut events = StorageVec::new();
    let mut supress_default = None;

    match msg {
        WM_ERASEBKGND => {
            // redraw the background
            log::info!("Redrawing background...");

            let surface = unwrap_surface!(surface, WM_ERASEBKGND);
            let wsurface = surface.as_win32().unwrap();
            unsafe { DefWindowProcA(wsurface.hwnd().as_ptr(), msg, wparam, lparam) };

            // draw the background brush
            let dc = unsafe { mem::transmute::<WPARAM, HDC>(wparam) };

            if let Some(background_brush) = wsurface.background_brush() {
                let mut rectangle: RECT = RECT {
                    left: 0,
                    right: 0,
                    bottom: 0,
                    top: 0,
                };
                unsafe { GetWindowRect(wsurface.hwnd().as_ptr(), &mut rectangle) };
                let (width, height) = (
                    rectangle.right - rectangle.left,
                    rectangle.bottom - rectangle.top,
                );

                // TODO: clean this up
                let hpen = unsafe { wingdi::CreatePen(wingdi::PS_SOLID as _, 0, 0) };
                unsafe { wingdi::SelectObject(dc, background_brush.as_ptr() as *mut _) };
                unsafe { wingdi::SelectObject(dc, hpen as *mut _) };
                unsafe { wingdi::SetBkMode(dc, wingdi::OPAQUE as _) };
                unsafe { wingdi::Rectangle(dc, 0, 0, width as _, height as _) };
                unsafe { wingdi::DeleteObject(hpen as *mut _) };
            }

            // TODO: draw the border

            supress_default = Some(1);
        }
        WM_CLOSE => {
            // destroy the window
            let surface = surface.unwrap().as_win32().unwrap();
            log::trace!("C function call: DestroyWindow({:p})", surface.hwnd());
            unsafe { DestroyWindow(surface.hwnd().as_ptr()) };
        }
        WM_DESTROY => {
            // post the quit message
            log::trace!("C function call: PostQuitMessage(0)");
            unsafe { PostQuitMessage(0) };
        }
        WM_SIZE => {
            // get the new size
            let lparam = lparam as UINT;
            let width = LOWORD(lparam) as u32;
            let height = HIWORD(lparam) as u32;

            // emit a changed event
            let surface = unwrap_surface!(surface, WM_SIZE);
            let (current_width, current_height) = surface.size();
            surface.set_size_no_backend(width as _, height as _);
            events.push(Event::new(
                EventType::Resized {
                    old: size2(current_width, current_height),
                    new: size2(width, height),
                },
                Some(surface.id()),
            ));
        }
        WM_MOVE => {
            // get the point
            let lparam = lparam as UINT;
            let x = LOWORD(lparam) as i32;
            let y = HIWORD(lparam) as i32;

            // emit a moved event
            let surface = unwrap_surface!(surface, WM_MOVE);
            let (current_x, current_y) = surface.location();
            surface.set_location_no_backend(x, y);
            events.push(Event::new(
                EventType::Moved {
                    old: point2(current_x, current_y),
                    new: point2(x, y),
                },
                Some(surface.id()),
            ));
        }
        WM_LBUTTONUP | WM_LBUTTONDOWN | WM_MBUTTONUP | WM_MBUTTONDOWN | WM_RBUTTONUP
        | WM_RBUTTONDOWN => {
            // figure out where the click happened
            let x = GET_X_LPARAM(lparam) as i32;
            let y = GET_Y_LPARAM(lparam) as i32;
            let pt = point2(x, y);

            // emit the proper event
            let surface = unwrap_surface!(surface, mouse);
            events.push(Event::new(
                match msg {
                    WM_LBUTTONUP => EventType::MouseUp(pt, MouseButton::Button1),
                    WM_LBUTTONDOWN => EventType::MouseDown(pt, MouseButton::Button1),
                    WM_MBUTTONUP => EventType::MouseUp(pt, MouseButton::Button2),
                    WM_MBUTTONDOWN => EventType::MouseDown(pt, MouseButton::Button2),
                    WM_RBUTTONUP => EventType::MouseUp(pt, MouseButton::Button3),
                    WM_RBUTTONDOWN => EventType::MouseDown(pt, MouseButton::Button3),
                    _ => unreachable!(),
                },
                Some(surface.id()),
            ));
        }
        WM_KEYDOWN | WM_KEYUP | WM_SYSKEYDOWN | WM_SYSKEYUP => {
            let key: usize = wparam;
            let key_ty = KeyType::from_win32(key);
            let surface = unwrap_surface!(surface, keyboard);

            // query keyboard state for function keys
            #[inline]
            fn query_key_state<F>(info: &mut KeyInfo, vk_code: c_int, mut doer: F)
            where
                F: FnMut(&mut KeyInfo),
            {
                const ACTIVATED: USHORT = 0x8000;
                if unsafe { GetKeyState(vk_code) } as USHORT & ACTIVATED != 0 {
                    doer(info);
                }
            }

            // get the current mouse position
            let mut mouse_pos: MaybeUninit<POINT> = MaybeUninit::zeroed();
            if unsafe { GetCursorPos(mouse_pos.as_mut_ptr()) } == 0 {
                return Err(crate::win32error("GetCursorPos"));
            }
            if unsafe {
                ScreenToClient(
                    surface.as_win32().unwrap().hwnd().as_ptr(),
                    mouse_pos.as_mut_ptr(),
                )
            } == 0
            {
                return Err(crate::win32error("ScreenToClient"));
            }
            let mouse_pos = unsafe { mouse_pos.assume_init() };
            let mouse_pos = point2(mouse_pos.x as _, mouse_pos.y as _);

            let mut key_info = KeyInfo::new(key_ty);
            query_key_state(&mut key_info, VK_SHIFT, |ki| ki.set_shift(true));
            query_key_state(&mut key_info, VK_CONTROL, |ki| ki.set_ctrl(true));
            query_key_state(&mut key_info, VK_MENU, |ki| ki.set_alt(true));
            events.push(Event::new(
                match msg {
                    WM_KEYUP | WM_SYSKEYUP => EventType::KeyUp(key_info, mouse_pos),
                    _ => EventType::KeyDown(key_info, mouse_pos),
                },
                Some(surface.id()),
            ));
        }
        WM_PAINT => {
            // create a paint structure
            let surface = unwrap_surface!(surface, WM_PAINT);
            let wsurface = surface.as_win32().unwrap();

            let mut paint_structure: MaybeUninit<PAINTSTRUCT> = MaybeUninit::uninit();
            if unsafe { BeginPaint(wsurface.hwnd().as_ptr(), paint_structure.as_mut_ptr()) }
                .is_null()
            {
                return Err(crate::win32error("BeginPaint"));
            }

            // create the painter
            let paint_structure = unsafe { paint_structure.assume_init() };
            wsurface.put_painter(
                NonNull::new(paint_structure.hdc)
                    .ok_or_else(|| crate::Error::Win32(crate::Win32Error::DCIsNull))?,
            );

            // create a paint event
            let pev = Event::new(
                EventType::Paint(Graphics::new(wsurface.graphics_internal()?)),
                Some(surface.id()),
            );
            // run the event loop on the paint event
            if !runtime.peeker_loop(&runtime.peekers(), &pev)? {
                log::error!("Currently cannot exit event loop during paint event handler");
            }

            wsurface.take_painter();

            unsafe { EndPaint(wsurface.hwnd().as_ptr(), &paint_structure) };
        }
        _ => log::debug!("Non-handled event: {}", msg),
    }

    Ok((events, supress_default))
}
