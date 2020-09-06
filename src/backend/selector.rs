// MIT/Apache2 License

use super::Backend;

#[cfg(unix)]
use super::x11::x11_backend_selector;

/// Selector function for backends.
pub(crate) type BackendSelector = &'static dyn Fn() -> Option<Backend>;

const BACKEND_SELECTORS: &[BackendSelector] = &[
    #[cfg(unix)]
    &x11_backend_selector,
];

#[inline]
pub(crate) fn select_backend() -> Option<Backend> {
    // go backwards
    BACKEND_SELECTORS.iter().rev().find_map(|sel| sel())
}
