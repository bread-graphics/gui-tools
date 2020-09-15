// MIT/Apache2 License

#[path = "common/deadlock.rs"]
mod deadlock;

use deadlock::deadlock_detector;
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
        SurfaceInitialization::new(None, StartingPoint::Center, 300, 300, "Example: Squares");
    properties.event_mask.push(EventTypeMask::Paint);

    runtime.create_surface(properties)?;

    runtime.add_peeker_owned(move |r, event| {
        if let EventType::Paint(ref g) = event.ty() {
            let surface = r.surface_at(event.sender().unwrap()).unwrap();
            let (width, height) = surface.size();

            g.set_color(colors::BLUE)?;
            for i in 0..(width as i32 / 30) + 1 {
                for j in 0..(height as i32 / 30) + 1 {
                    if (i + j) & 1 == 0 {
                        g.draw_rectangle(i * 30, j * 30, 30, 30)?;
                    } else {
                        g.fill_rectangle(i * 30, j * 30, 30, 30)?;
                    }
                }
            }
        }

        Ok(EventLoopAction::Continue)
    });

    runtime.run()
}
