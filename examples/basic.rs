// MIT/Apache2 License

use gui_tools::{prelude::*, Color, DisplaySum, Event, FillRule, Result, WindowProps};

fn main() -> Result {
    env_logger::init();

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

    dpy.run(|dpy, ev| { 
        println!("{:?}", &ev); 
        match ev {
            Event::Paint(window) => dpy.draw(window, |painter| {
                painter.set_stroke(Color::BLACK)?;
                painter.set_fill(FillRule::SolidColor(Color::new(0.0, 0.0, 1.0, 1.0).unwrap()))?;
                painter.set_line_width(5)?;
                
                painter.fill_ellipse(50, 50, 150, 150)?;
                painter.draw_ellipse(50, 50, 150, 150)?;
                painter.flush()?;

                Ok(())
            })?,
            _ => {},
        }
        Ok(()) 
    })?;

    Ok(())
}
