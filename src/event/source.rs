// MIT/Apache2 License

use super::Event;
use storagevec::StorageVec;

/// A source for events.
pub trait EventSource {
    fn pending(&self) -> bool {
        self.poll().is_none()
    }
    fn poll(&self) -> Option<StorageVec<Event, 5>>;
}
