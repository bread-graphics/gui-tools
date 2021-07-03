// MIT/Apache2 License

use super::{cvt_window_r, BreadxDisplay, WM_DELETE_WINDOW};
use crate::event::Event;
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
                // TODO: figure out a way not to emit two events
                Ok(TinyVec::Heap(vec![
                    Event::Move {
                        window,
                        x: cc.x.into(),
                        y: cc.y.into(),
                    },
                    Event::Resize {
                        window,
                        width: cc.width.into(),
                        height: cc.height.into(),
                    },
                ]))
            }
            _ => Ok(Default::default()),
        }
    }
}

#[inline]
fn singleton(ev: Event) -> TinyVec<[Event; 1]> {
    TinyVec::Inline(ArrayVec::from([ev]))
}
