// MIT/Apache2 License

use super::X11Monitor;
use core::{mem::MaybeUninit, ptr::NonNull, slice};
use cty::{c_int, c_long, c_void};
#[cfg(feature = "alloc")]
use hashbrown::HashSet;
use storagevec::StorageVec;
#[cfg(not(feature = "alloc"))]
use tinyvec::ArraySet;
use x11nas::xlib::{self, Visual, XVisualInfo};

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum X11VisualType {
    StaticGrayscale,
    Grayscale,
    StaticColor,
    PseudoColor,
    TrueColor,
    DirectColor,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct X11Visual {
    // sort by depth, then type
    depth: i32,
    ty: X11VisualType,

    bits_per_rgb: i32,
    color_mask: (u32, u32, u32),
    visual: NonNull<Visual>,
}

impl X11Visual {
    #[inline]
    pub fn visual(&self) -> &NonNull<Visual> {
        &self.visual
    }

    #[inline]
    pub fn depth(&self) -> i32 {
        self.depth
    }

    fn from_x11_visual_info(info: &XVisualInfo) -> crate::Result<X11Visual> {
        log::trace!("Convert XVisualInfo into X11Visual: {:?}", info);

        if info.depth < 1 {
            return Err(crate::X11Error::BadVisualDepth(info.depth).into());
        }

        let mut visual = X11Visual {
            visual: match NonNull::new(info.visual) {
                Some(vis) => vis,
                None => return Err(crate::X11Error::BadVisualPointer.into()),
            },
            ty: match info.class {
                xlib::StaticGray => X11VisualType::StaticGrayscale,
                xlib::GrayScale => X11VisualType::Grayscale,
                xlib::StaticColor => X11VisualType::StaticColor,
                xlib::PseudoColor => X11VisualType::PseudoColor,
                xlib::TrueColor => X11VisualType::TrueColor,
                xlib::DirectColor => X11VisualType::DirectColor,
                _ => return Err(crate::X11Error::BadVisualColorType(info.class).into()),
            },
            color_mask: (
                info.red_mask as u32,
                info.green_mask as u32,
                info.blue_mask as u32,
            ),
            depth: info.depth,
            bits_per_rgb: info.bits_per_rgb,
        };

        // the masks are irrelevant if the color type isn't true or direct
        match visual.ty {
            X11VisualType::TrueColor | X11VisualType::DirectColor => (),
            _ => {
                visual.color_mask = (0, 0, 0);
            }
        }

        Ok(visual)
    }

    pub(crate) fn setup_monitor(screen: &mut X11Monitor) -> crate::Result<()> {
        log::debug!("Setting up monitor for screen #{}", screen.screen_id());

        let mut visual_count: MaybeUninit<c_int> = MaybeUninit::uninit();
        log::trace!("Unsafe code: using MaybeUninit::uninit().assume_init() to gradually initialize the visual template");
        // SAFETY: X11 only uses the Screen field, due to the VisualScreenMask
        let mut visual_template = XVisualInfo {
            screen: screen.screen_id(),
            ..unsafe { MaybeUninit::uninit().assume_init() }
        };

        // SAFETY: Calls a C function; the result is checked
        let visual_ptr: *const XVisualInfo = unsafe {
            xlib::XGetVisualInfo(
                screen.display().as_ptr(),
                xlib::VisualScreenMask,
                &mut visual_template,
                visual_count.as_mut_ptr(),
            )
        };

        if visual_ptr.is_null() {
            return Err(crate::X11Error::BadGetVisualInfo.into());
        }

        // convert the visual pointer to a slice in order to take advantage of iterators
        // SAFETY: The x11 implementation makes sure visual_count is the number of XVisualInfos
        //         returned by the XGetVisualInfo call.
        let visuals = unsafe {
            slice::from_raw_parts(visual_ptr, MaybeUninit::assume_init(visual_count) as usize)
        };

        // also, get the default visual for the monitor
        // SAFETY: calls a C function
        let default_visual: *mut Visual =
            unsafe { xlib::XDefaultVisual(screen.display().as_ptr(), screen.screen_id()) };
        let default_visual = match NonNull::new(default_visual) {
            Some(dv) => dv,
            None => return Err(crate::X11Error::BadDefaultVisual.into()),
        };

        let mut default_visual_index: Option<usize> = None;
        let mut rgba_visual_index: Option<usize> = None;

        let mut res: StorageVec<Self, 6> = visuals
            .iter()
            .enumerate()
            .map(|(i, visual_info)| {
                // if the visual ID is the default visual ID, set the index
                // SAFETY: I don't even know why as_ref is unsafe in the first place, it's not like it's null
                if unsafe { default_visual.as_ref() }.visualid == visual_info.visualid {
                    default_visual_index = Some(i);
                }

                // also figure out if this is an RGBA visual type
                if rgba_visual_index.is_none() {
                    match (
                        visual_info.depth,
                        visual_info.red_mask,
                        visual_info.green_mask,
                        visual_info.blue_mask,
                    ) {
                        (32, 0xFF0000, 0x00FF00, 0x0000FF) => {
                            rgba_visual_index = Some(i);
                        }
                        _ => (),
                    }
                }

                let depth = visual_info.depth;
                Self::from_x11_visual_info(visual_info)
            })
            .collect::<crate::Result<_>>()?;

        // sort the resulting list
        res.sort();

        // set up the monitor
        screen.visuals = Some(res);
        screen.default_visual =
            Some(default_visual_index.expect("Unable to find default visual in list"));
        screen.rgba_visual = rgba_visual_index;

        // note: X11 expects us to free the visual ptr list
        log::trace!("C function call: XFree({:p})", visual_ptr);
        unsafe { xlib::XFree(visual_ptr as *mut c_void) };

        Ok(())
    }
}
