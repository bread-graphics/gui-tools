// MIT/Apache2 License

/// Sum type representing a possible event emitted by the underlying GUI library.
#[derive(Debug)]
pub enum Event {
    /// The display is being closed; good time to deallocate resources.
    Closing,
}
