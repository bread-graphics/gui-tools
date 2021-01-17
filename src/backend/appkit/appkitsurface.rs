// MIT/Apache2 License

use super::CGFloat;
use crate::{
    event::{Event, EventType, EventTypeMask},
    geometry::Pixel,
    graphics::{Graphics, GraphicsInternal},
    runtime::{Runtime, RuntimeInternal},
    surface::{SurfaceBackend, SurfaceInitialization},
};
use conquer_once::Lazy;
use cty::c_void;
use euclid::Rect;
use objc::{
    class,
    declare::ClassDecl,
    msg_send,
    runtime::{id, Class, Object, Sel},
    sel, Encode, Encoding,
};

static WINDOW_CLASS: Lazy<&'static Class> = Lazy::new(|| {
    // parent class
    let parent = class!(NSWindow);
    let decl = ClassDecl::new("GuiToolsWindow", parent).unwrap();

    // display var
    decl.add_ivar::<*const c_void>("runtime");
    decl.add_method(sel!(drawRect), redrawer);
    decl.add_method(sel!(setRuntime), set_runtime);

    decl.register()
});

static VIEW_CLASS: Lazy<&'static Class> = Lazy::new(|| {
    let parent = class!(NSView);
    let decl = ClassDecl::new("GuiToolsView", parent).unwrap();

    decl.add_ivar::<*const c_void>("runtime");
    decl.add_method(sel!(drawRect), redrawer);
    decl.add_method(sel!(setRuntime), set_runtime);

    decl.register()
});

// we need something with the layout of an NSRect
#[repr(C)]
struct NSRect {
    pt: NSPoint,
    sz: NSSize,
}

#[repr(C)]
struct NSPoint {
    x: CGFloat,
    y: CGFloat,
}

#[repr(C)]
struct NSSize {
    width: CGFloat,
    height: CGFloat,
}

unsafe impl Encode for NSRect {
    #[inline]
    fn encode() -> Encoding {
        Encoding::from_str("dddd")
    }
}

extern "C" fn set_runtime(this: &Object, ptr: *const c_void, _cmd: Sel) {
    this.set_ivar::<*const c_void>("runtime", ptr);
}

extern "C" fn redrawer(this: &Object, dirty_rect: NSRect, _cmd: Sel) {
    // get the runtime
    let runtime: &*const c_void = this.get_ivar("runtime");
    let runtime = Runtime::from_ptr(*runtime as *const _);

    if let Err(e) = redrawer_internal(this, dirty_rect, &runtime) {
        runtime.as_appkit().unwrap().store_error(e);
    }

    mem::forget(runtime);
}

fn redrawer_internal(this: &Object, dirty_rect: NSRect, runtime: &Runtime) -> crate::Result {
    // get the window
    let wid = this as *const _ as *const () as usize;
    let surface = runtime.surface_at(wid).unwrap();

    // run the draw circuit
    let paint_event = Event::new(
        EventType::Paint(Graphics::new(
            surface.as_appkit().unwrap().graphics_internal()?,
        )),
        Some(wid),
    );
    let peekers = runtime.peekers().clone();
    runtime.peeker_loop(&peekers, &paint_event)?;

    Ok(())
}

pub struct AppkitSurface {
    toplevel: bool,
    sys: id,
    runtime_ptr: *const RuntimeInternal,
}

impl AppkitSurface {
    #[inline]
    pub fn new(runtime: &Runtime, init: &SurfaceInitialization) -> crate::Result<Self> {
        let aruntime = runtime.as_appkit().unwrap();
        let runtime_ptr = runtime.clone().into_ptr();
        let (internal, toplevel) = match init.parent {
            Some(parent) => {
                let class = &*VIEW_CLASS;
                let parent = runtime.surface_at(parent).unwrap();
                let (x, y) = init.starting_point.to_x_y(
                    init.width,
                    init.height,
                    parent.width(),
                    parent.height(),
                );
                let frame = NSRect {
                    pt: NSPoint {
                        x: x.into(),
                        y: y.into(),
                    },
                    sz: NSSize {
                        width: init.width.into(),
                        height: init.height.into(),
                    },
                };
                let win: id = msg_send![class initWithFrame:frame];
                (win, false)
            }
            None => {
                let class = &*WINDOW_CLASS;
                let monitor = runtime.default_monitor().unwrap();
                let (x, y) = init.starting_point.to_x_y(
                    init.width,
                    init.height,
                    monitor.width(),
                    monitor.height(),
                );
                let frame = NSRect {
                    pt: NSPoint {
                        x: x.into(),
                        y: y.into(),
                    },
                    sz: NSSize {
                        width: init.width.into(),
                        height: init.height.into(),
                    },
                };
                let win: id = msg_send![class initWithContentRect:frame styleMask: NSWindowStyleMaskTitled | NSWindowStyleMaskClosable | NSWindowStyleMaskMiniaturizable | NSWindowStyleMaskResizable defer: YES];
                (win, true)
            }
        };

        let _: () = msg_send![win setRuntime:runtime_ptr];
        // TODO: set other aspects of the SurfaceInitialization

        Ok(Self {
            toplevel,
            sys: internal,
            runtime_ptr,
        })
    }
}

impl Drop for AppkitSurface {
    #[inline]
    fn drop(&mut self) {
        let _: id = msg_send![self.sys dealloc];
        Runtime::from_ptr(self.runtime_ptr);
    }
}

impl SurfaceBackend for AppkitSurface {
    #[inline]
    fn id(&self) -> usize {
        self.sys as *const () as usize
    }

    #[inline]
    fn set_event_mask(&self, _event_mask: &[EventTypeMask]) -> crate::Result {
        Ok(())
    }
}
