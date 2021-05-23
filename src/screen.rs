// MIT/Apache2 License

use crate::util::DebugContainer;
use std::{ops::Range, vec::IntoIter as VecIter};

/// A logical screen, often representing a monitor or collection of monitors.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Screen {
    // this is often either an index into an array or a pointer. in any case, a usize
    // fits both of these use cases
    inner: usize,
}

impl Screen {
    #[inline]
    pub fn from_raw(inner: usize) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn into_raw(self) -> usize {
        self.inner
    }
}

/// An iterator over the screens provided by a display.
#[derive(Debug)]
pub struct ScreenIter<'a> {
    inner: Repr<'a>,
}

#[derive(Debug)]
enum Repr<'a> {
    Range(Range<usize>),
    Vector(VecIter<usize>),
    Other(DebugContainer<Box<dyn Iterator<Item = usize> + Send + 'a>>),
}

impl<'a> ScreenIter<'a> {
    #[inline]
    pub fn from_range(begin_at: usize, end_before: usize) -> Self {
        Self {
            inner: Repr::Range(begin_at..end_before),
        }
    }

    #[inline]
    pub fn from_vector(v: Vec<usize>) -> Self {
        Self {
            inner: Repr::Vector(v.into_iter()),
        }
    }

    #[inline]
    pub fn from_iterator<I: IntoIterator<Item = usize>>(i: I) -> Self
    where
        I::IntoIter: ExactSizeIterator + Send + 'a,
    {
        Self {
            inner: Repr::Other(DebugContainer::new(Box::new(i.into_iter()))),
        }
    }
}

impl<'a> Iterator for ScreenIter<'a> {
    type Item = Screen;

    #[inline]
    fn next(&mut self) -> Option<Screen> {
        match &mut self.inner {
            Repr::Range(r) => r.next().map(Screen::from_raw),
            Repr::Vector(v) => v.next().map(Screen::from_raw),
            Repr::Other(i) => i.next().map(Screen::from_raw),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.inner {
            Repr::Range(r) => r.size_hint(),
            Repr::Vector(v) => v.size_hint(),
            Repr::Other(i) => i.size_hint(),
        }
    }
}

// as per "from_iterator"'s type bounds, ScreenIter::Other implicitly derives ExactSizeIterator
impl<'a> ExactSizeIterator for ScreenIter<'a> {}
