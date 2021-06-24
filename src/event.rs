// MIT/Apache2 License

/// Sum type representing a possible event emitted by the underlying GUI library.
#[derive(Debug)]
pub enum Event {
    /// The display is being closed; good time to deallocate resources.
    Closing,
    /// Special event that tells the `Display` to quit.
    Quit,
}

impl Event {
    #[inline]
    pub fn is_quit_event(&self) -> bool {
        matches!(self, Event::Quit)
    }
}
