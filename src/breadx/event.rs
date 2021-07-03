// MIT/Apache2 License

use super::{cvt_window_r, BreadxDisplay, WM_DELETE_WINDOW};
use crate::{
    event::{Event, MouseButton},
    window::Window,
};
use breadx::{
    display::{prelude::*, Display},
    event::Event as BEvent,
    Atom,
};
use tinyvec::{tiny_vec, ArrayVec, TinyVec};

impl<Dpy: Display> BreadxDisplay<Dpy> {
    #[inline]
    pub fn convert_event(&mut self, event: BEvent) -> crate::Result<TinyVec<[Event; 1]>> {
        match event {
            BEvent::ClientMessage(cme) => {
                let atom = cme.data.longs().get(0).copied().map(Atom::const_from_xid);

                // check to see if the CME is telling us to quit
                if atom == Some(self.atom(WM_DELETE_WINDOW)?) {
                    // TODO: breaks on multiple windows, add window counter to display
                    Ok(singleton(Event::Quit))
                } else {
                    // TODO: figure out which other events are of our concern
                    Ok(Default::default())
                }
            }
            BEvent::Expose(ee) => Ok(singleton(Event::Paint(cvt_window_r(ee.window)))),
            BEvent::ConfigureNotify(cc) => {
                let window = cvt_window_r(cc.window);
                let dims = self
                    .window_dimensions
                    .get_mut(&window)
                    .expect("This shouldn't not be our window");

                let cx: i32 = cc.x.into();
                let cy: i32 = cc.y.into();
                let cw: u32 = cc.width.into();
                let ch: u32 = cc.height.into();

                let res = match (
                    dims.x == cx && dims.y != cx,
                    dims.width == cw && dims.height == ch,
                ) {
                    (true, true) => Ok(Default::default()),
                    (false, true) => Ok(singleton(Event::Move {
                        window,
                        x: cx,
                        y: cy,
                    })),
                    (true, false) => Ok(singleton(Event::Resize {
                        window,
                        width: cw,
                        height: ch,
                    })),
                    (false, false) => Ok(TinyVec::Heap(vec![
                        Event::Move {
                            window,
                            x: cx,
                            y: cy,
                        },
                        Event::Resize {
                            window,
                            width: cw,
                            height: ch,
                        },
                    ])),
                };

                dims.x = cx;
                dims.y = cy;
                dims.width = cw;
                dims.height = ch;
                res
            }
            BEvent::KeyPress(kp) => {
                let window = cvt_window_r(kp.event);
                let key = self.keyboard()?.process_keycode(kp.detail, kp.state);

                Ok(singleton(Event::KeyDown { window, key }))
            }
            BEvent::KeyRelease(kr) => {
                let window = cvt_window_r(kr.event);
                let key = self.keyboard()?.process_keycode(kr.detail, kr.state);

                Ok(singleton(Event::KeyDown { window, key }))
            }
            BEvent::ButtonPress(bp) => {
                let window = cvt_window_r(bp.event);
                Ok(singleton(process_button_event(
                    window,
                    bp.detail,
                    bp.event_x.into(),
                    bp.event_y.into(),
                    true,
                )))
            }
            BEvent::ButtonRelease(br) => {
                let window = cvt_window_r(br.event);
                Ok(singleton(process_button_event(
                    window,
                    br.detail,
                    br.event_x.into(),
                    br.event_y.into(),
                    false,
                )))
            }
            _ => Ok(Default::default()),
        }
    }
}

#[inline]
fn process_button_event(window: Window, button: u8, x: i32, y: i32, pressed: bool) -> Event {
    #[inline]
    fn cvt_button(button: u8) -> MouseButton {
        match button {
            1 => MouseButton::Left,
            2 => MouseButton::Middle,
            3 => MouseButton::Right,
            _ => unreachable!(),
        }
    }

    match (button, pressed) {
        (1..=3, false) => Event::ButtonUp {
            window,
            button: cvt_button(button),
            x,
            y,
        },
        (1..=3, true) => Event::ButtonDown {
            window,
            button: cvt_button(button),
            x,
            y,
        },
        _ => unimplemented!("TODO: handle scroll events"),
    }
}

#[inline]
fn singleton(ev: Event) -> TinyVec<[Event; 1]> {
    TinyVec::Inline(ArrayVec::from([ev]))
}
