// MIT/Apache2 License

use super::{cvt_window_r, YawwDisplay};
use crate::event::{Event, MouseButton};
use yaww::{Event as YawwEvent, MouseButton as YawwMouseButton};

#[inline]
pub(crate) fn cvt_event<'evh>(
    yaww: &YawwDisplay<'evh>,
    event: YawwEvent,
) -> crate::Result<Option<Event>> {
    Ok(Some(match event {
        YawwEvent::Quit => Event::Quit,
        YawwEvent::Move { window, x, y } => Event::Move {
            window: cvt_window_r(window),
            x,
            y,
        },
        YawwEvent::Size {
            window,
            width,
            height,
            ..
        } => Event::Resize {
            window: cvt_window_r(window),
            width: width as u32,
            height: height as u32,
        },
        YawwEvent::SetFocus { window, .. } => Event::Activate(cvt_window_r(window)),
        YawwEvent::KillFocus { window, .. } => Event::Deactivate(cvt_window_r(window)),
        YawwEvent::Paint { window, dc } => {
            // store the DC in the display
            yaww.store_dc(window, dc);
            Event::Paint(cvt_window_r(window))
        }
        YawwEvent::KeyDown { window, key, .. } => Event::KeyDown {
            window: cvt_window_r(window),
            key,
        },
        YawwEvent::KeyUp { window, key, .. } => Event::KeyUp {
            window: cvt_window_r(window),
            key,
        },
        YawwEvent::ButtonDown {
            window,
            button,
            x,
            y,
        } => {
            return Ok(
                cvt_mouse_button(button).map(move |button| Event::ButtonDown {
                    window: cvt_window_r(window),
                    button,
                    x,
                    y,
                }),
            )
        }
        YawwEvent::ButtonUp {
            window,
            button,
            x,
            y,
        } => {
            return Ok(cvt_mouse_button(button).map(move |button| Event::ButtonUp {
                window: cvt_window_r(window),
                button,
                x,
                y,
            }))
        }
        YawwEvent::MouseMove { window, x, y } => Event::MouseMove {
            window: cvt_window_r(window),
            x,
            y,
        },
        _ => return Ok(None),
    }))
}

#[inline]
fn cvt_mouse_button(mb: YawwMouseButton) -> Option<MouseButton> {
    Some(match mb {
        YawwMouseButton::Left => MouseButton::Left,
        YawwMouseButton::Middle => MouseButton::Middle,
        YawwMouseButton::Right => MouseButton::Right,
        _ => return None,
    })
}
