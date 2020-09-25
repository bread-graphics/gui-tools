// MIT/Apache2 License

use super::Backend;

#[cfg(target_os = "macos")]
use super::appkit::appkit_backend_selector;
#[cfg(windows)]
use super::win32::{win32_backend_selector_commctrl, win32_backend_selector_no_commctrl};
#[cfg(unix)]
use super::x11::x11_backend_selector;

use storagevec::StorageVec;

/// Selector function for backends.
pub(crate) type BackendSelector = &'static dyn Fn() -> Option<Backend>;

const BACKEND_SELECTORS: &[BackendSelector] = &[
    #[cfg(windows)]
    &win32_backend_selector_no_commctrl,
    #[cfg(windows)]
    &win32_backend_selector_commctrl,
    #[cfg(target_os = "macos")]
    &appkit_backend_selector,
    #[cfg(unix)]
    &x11_backend_selector,
];

#[inline]
pub(crate) fn select_backend() -> Option<Backend> {
    // go backwards
    BACKEND_SELECTORS.iter().rev().find_map(|sel| sel())
}
