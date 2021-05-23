// MIT/Apache2 License

use crate::display::Display;

pub struct Event<'a> {
    assoc_display: Option<&'a mut dyn Display>,
}
