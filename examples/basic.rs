// MIT/Apache2 License

use gui_tools::{prelude::*, Color, DisplaySum, FillRule, Result, WindowProps};

fn main() -> Result {
    let mut dpy = DisplaySum::create()?;
    let scr = dpy.default_screen()?;
    let root = dpy.toplevel_window(scr)?;
    let window = dpy.create_window(
        0,
        0,
        600,
        480,
        root,
        WindowProps {
            title: Some("Hello world!".to_string()),
            background: Some(FillRule::SolidColor(Color::WHITE)),
            border_color: Some(Color::BLACK),
            ..Default::default()
        },
    )?;
    dpy.set_window_visibility(window, true)?;

    dpy.run(|dpy, ev| Ok(()))?;

    Ok(())
}
