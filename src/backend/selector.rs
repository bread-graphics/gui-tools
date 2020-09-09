// MIT/Apache2 License

use super::Backend;

#[cfg(windows)]
use super::win32::{win32_backend_selector_commctrl, win32_backend_selector_no_commctrl};
#[cfg(target_os = "linux")]
use super::x11::x11_backend_selector;

/// Selector function for backends.
pub(crate) type BackendSelector = &'static dyn Fn() -> Option<Backend>;

const BACKEND_SELECTORS: &[BackendSelector] = &[
    #[cfg(target_os = "linux")]
    &x11_backend_selector,
    #[cfg(windows)]
    &win32_backend_selector_no_commctrl,
    #[cfg(windows)]
    &win32_backend_selector_commctrl,
];

#[inline]
pub(crate) fn select_backend() -> Option<Backend> {
    // go backwards
    BACKEND_SELECTORS.iter().rev().find_map(|sel| sel())
}
