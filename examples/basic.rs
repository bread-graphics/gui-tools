// MIT/Apache2 License

#[path = "common/deadlock.rs"]
mod deadlock;

use deadlock::deadlock_detector;
use euclid::{Angle, point2};
use gui_tools::{
    color::colors,
    error::Result,
    event::{EventLoopAction, EventType, EventTypeMask},
    runtime::Runtime,
    surface::{StartingPoint, SurfaceInitialization},
};
use std::env;

fn main() -> Result<()> {
    env::set_var("RUST_LOG", "gui_tools=info");
    env_logger::init();

    deadlock_detector();

    let runtime = Runtime::new()?;
    let mut properties =
        SurfaceInitialization::new(None, StartingPoint::Center, 300, 300, "Example: Basic");
    properties.event_mask.extend(vec![
        EventTypeMask::MouseUp,
        EventTypeMask::MouseDown,
        EventTypeMask::Moved,
        EventTypeMask::KeyDown,
        EventTypeMask::KeyUp,
        EventTypeMask::Paint,
    ]);
    let surface = runtime.create_surface(properties)?;

    runtime.add_peeker_owned(move |r, event| {
        println!("{:?}", event);

        // if this is a graphical event, draw a line
        if let EventType::Paint(ref g) = event.ty() {
            g.set_color(colors::GREEN)?;
            g.set_line_width(4)?;
            g.draw_lines(&[
                point2(60, 10),
                point2(110, 50),
                point2(110, 50),
                point2(10, 50),
                point2(10, 50),
                point2(60, 10),
            ])?;

            g.set_color(colors::RED)?;
            g.fill_arc(150, 150, 60, 80, Angle::degrees(0.0), Angle::degrees(270.0))?;
            g.set_color(colors::BLACK)?;
//            g.draw_ellipse(150, 150, 60, 80)?;
        } else if let EventType::MouseDown(_, _) = event.ty() {
            r.surface_at(surface)
                .unwrap()
                .set_background_color(colors::BLUE)?;
        }

        Ok(EventLoopAction::Continue)
    });

    runtime.run()
}
