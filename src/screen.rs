// MIT/Apache2 License

use std::{ops::Range, vec::IntoIter as VecIter};

/// A logical screen, consisting of an area of screen space.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Screen {
    /// The screen is usually either an index into an array or a pointer. In either case, a `usize`
    /// fits.
    screen: usize,
}

impl Screen {
    /// Get a screen from a raw `usize`.
    #[inline]
    pub fn from_raw(screen: usize) -> Screen {
        Screen { screen }
    }

    /// Get the raw `usize` out of this structure.
    #[inline]
    pub fn into_raw(self) -> usize {
        self.screen
    }
}

/// An iterator over a set of screens.
pub struct ScreenIter<'iter> {
    inner: Impl<'iter>,
}

enum Impl<'iter> {
    /// Just a range from two numbers.
    Range(Range<usize>),
    /// An iterator over a vector.
    Vector(VecIter<usize>),
    /// A generic iterator.
    Generic(Box<dyn Iterator<Item = usize> + 'iter>),
}

impl<'iter> ScreenIter<'iter> {
    /// Create a `ScreenIter` over a range from `start` to `end`, `end` exclusive.
    #[inline]
    pub fn range(start: usize, end: usize) -> ScreenIter<'iter> {
        ScreenIter {
            inner: Impl::Range(start..end),
        }
    }

    /// Create a `ScreenIter` using another iterator.
    #[inline]
    pub fn from_iterator<I: IntoIterator<Item = usize>>(iter: I) -> ScreenIter<'iter>
    where
        I::IntoIter: ExactSizeIterator + 'iter,
    {
        ScreenIter {
            inner: Impl::Generic(Box::new(iter.into_iter())),
        }
    }
}

impl<'iter> From<Vec<usize>> for ScreenIter<'iter> {
    /// Create a `ScreenIter` based on a vector full of `usize`s.
    #[inline]
    fn from(vector: Vec<usize>) -> ScreenIter<'iter> {
        ScreenIter {
            inner: Impl::Vector(vector.into_iter()),
        }
    }
}

impl<'iter> Iterator for ScreenIter<'iter> {
    type Item = Screen;

    #[inline]
    fn next(&mut self) -> Option<Screen> {
        match &mut self.inner {
            Impl::Range(r) => r.next().map(Screen::from_raw),
            Impl::Vector(v) => v.next().map(Screen::from_raw),
            Impl::Generic(g) => g.next().map(Screen::from_raw),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.inner {
            Impl::Range(r) => r.size_hint(),
            Impl::Vector(v) => v.size_hint(),
            Impl::Generic(g) => g.size_hint(),
        }
    }

    // Most iterator operations are implemented in terms of "next", "nth", "fold", or "try_fold". In order to
    // offset the cost of doing a branch prediction every time "next()" is called, we implement these operations
    // in order to forward their implementations to the inner type instead of running "next()" several times
    // note that we can't do "try_fold" because it uses nightly features (try_trait), so we just have to eat
    // the cost of that

    #[inline]
    fn nth(&mut self, index: usize) -> Option<Screen> {
        match &mut self.inner {
            Impl::Range(r) => r.nth(index).map(Screen::from_raw),
            Impl::Vector(v) => v.nth(index).map(Screen::from_raw),
            Impl::Generic(g) => g.nth(index).map(Screen::from_raw),
        }
    }

    #[inline]
    fn fold<B, F: FnMut(B, Screen) -> B>(self, init: B, mut closure: F) -> B {
        let closure = move |accum, item| closure(accum, Screen::from_raw(item));

        match self.inner {
            Impl::Range(r) => r.fold(init, closure),
            Impl::Vector(v) => v.fold(init, closure),
            Impl::Generic(g) => g.fold(init, closure),
        }
    }
}

// from_iterator() is parameterized by ExactSizeIterator, so this is sound
impl<'iter> ExactSizeIterator for ScreenIter<'iter> {}
