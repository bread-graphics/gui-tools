// MIT/Apache2 License

#[path = "common/deadlock.rs"]
mod deadlock;

use deadlock::deadlock_detector;
use gui_tools::{color::{colors, Rgba}, error::Result,  runtime::Runtime, surface::{StartingPoint, SurfaceInitialization}};
use std::{env, thread};

fn create_runtime(bkgrnd: Rgba, name: &'static str) -> Result<Runtime> {
    let runtime = Runtime::new()?;
    let mut properties = SurfaceInitialization::new(None, StartingPoint::Center, 300, 300, name);
    properties.background_color = bkgrnd;
    let surface = runtime.create_surface(properties)?;
    Ok(runtime)
}

fn main() -> Result<()> {
    env::set_var("RUST_LOG", "gui_tools=info");
    env_logger::init();

    deadlock_detector();
    
    let r1 = create_runtime(colors::RED, "Red")?;
    let r2 = create_runtime(colors::BLUE, "Blue")?;

    let j1 = thread::spawn(move || r1.run());
    let j2 = thread::spawn(move || r2.run());
    j1.join().unwrap()?;
    j2.join().unwrap()?;

    Ok(())
}
