// MIT/Apache2 License

use gui_tools::{error::Result, surface::{Surface, SurfaceProperties}, runtime::Runtime};

fn main() -> Result<()> {
    let runtime = Runtime::new()?;
    let properties = SurfaceProperties::new(None, 0, 0, 200, 200);
    let surface = runtime.create_surface(properties)?;
    Ok(())
}

