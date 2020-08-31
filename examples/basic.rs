// MIT/Apache2 License

#[path = "common/deadlock.rs"]
mod deadlock;

use deadlock::deadlock_detector;
use gui_tools::{error::Result, event::{EventLoopAction, EventTypeMask}, surface::{Surface, SurfaceInitialization, StartingPoint}, runtime::Runtime};
use std::env;

fn main() -> Result<()> {
    env::set_var("RUST_LOG", "gui_tools=trace");
    env_logger::init();

    deadlock_detector();

    let runtime = Runtime::new()?;
    runtime.add_peeker(&|r, ev| { println!("{:?}", ev); Ok(EventLoopAction::Continue) });
    let mut properties = SurfaceInitialization::new(None, StartingPoint::Center, 300, 200, "Basic Example");
    properties.event_mask.extend(vec![EventTypeMask::Clicked]);
    let surface = runtime.create_surface(properties)?;

    runtime.run()
}

