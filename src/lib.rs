#![feature(specialization)]
#![feature(trusted_len)]

use std::cmp;
use std::iter::{DoubleEndedIterator, ExactSizeIterator, Iterator, IntoIterator};
use std::slice::Iter;

pub trait GetZipOption<A>
where
    A: Iterator
{
    fn zip_option<B: Iterator>(self, other: B) -> ZipOption<A, B>;
}

impl<'a, T> GetZipOption<Self> for Iter<'a, T> {
    fn zip_option<U: Iterator>(self, other: U) -> ZipOption<Self, U> {
	ZipOption::new(self, other.into_iter())
    }
}

#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct ZipOption<A, B> {
    a: A,
    b: B,
    // index and len are only used by the specialized version of zip
    index: usize,
    len: usize,
}
impl<A: Iterator, B: Iterator> ZipOption<A, B> {
    pub fn new(a: A, b: B) -> ZipOption<A, B> {
        ZipImpl::new(a, b)
    }
    fn super_nth(&mut self, mut n: usize) -> Option<(Option<A::Item>, Option<B::Item>)> {
        while let Some(x) = Iterator::next(self) {
            if n == 0 {
                return Some(x);
            }
            n -= 1;
        }
        None
    }
}

impl<A, B> Iterator for ZipOption<A, B>
where
    A: Iterator,
    B: Iterator,
{
    type Item = (Option<A::Item>, Option<B::Item>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        ZipImpl::next(self)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        ZipImpl::size_hint(self)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        ZipImpl::nth(self, n)
    }
}

impl<A, B> DoubleEndedIterator for ZipOption<A, B>
where
    A: DoubleEndedIterator + ExactSizeIterator,
    B: DoubleEndedIterator + ExactSizeIterator,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        ZipImpl::next_back(self)
    }
}

// Zip specialization trait
#[doc(hidden)]
trait ZipImpl<A, B> {
    type Item;
    fn new(a: A, b: B) -> Self;
    fn next(&mut self) -> Option<Self::Item>;
    fn size_hint(&self) -> (usize, Option<usize>);
    fn nth(&mut self, n: usize) -> Option<Self::Item>;
    fn next_back(&mut self) -> Option<Self::Item>
    where
        A: DoubleEndedIterator + ExactSizeIterator,
        B: DoubleEndedIterator + ExactSizeIterator;
}

// General Zip impl
#[doc(hidden)]
impl<A, B> ZipImpl<A, B> for ZipOption<A, B>
where
    A: Iterator,
    B: Iterator,
{
    type Item = (Option<A::Item>, Option<B::Item>);
    default fn new(a: A, b: B) -> Self {
        ZipOption {
            a,
            b,
            index: 0, // unused
            len: 0,   // unused
        }
    }

    #[inline]
    default fn next(&mut self) -> Option<Self::Item> {
        match (self.a.next(), self.b.next()) {
	    (None, None) => None,
	    (x, y) => Some((x, y))
	}
    }

    #[inline]
    default fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.super_nth(n)
    }

    #[inline]
    default fn next_back(&mut self) -> Option<Self::Item>
    where
        A: DoubleEndedIterator + ExactSizeIterator,
        B: DoubleEndedIterator + ExactSizeIterator,
    {
        let a_sz = self.a.len();
        let b_sz = self.b.len();
        if a_sz != b_sz {
            // Adjust a, b to equal length
            if a_sz > b_sz {
                for _ in 0..a_sz - b_sz {
                    self.a.next_back();
                }
            } else {
                for _ in 0..b_sz - a_sz {
                    self.b.next_back();
                }
            }
        }

        match (self.a.next_back(), self.b.next_back()) {
            (None, None) => None,
            (x, y) => Some((x, y)),
        }
    }

    #[inline]
    default fn size_hint(&self) -> (usize, Option<usize>) {
        let (a_lower, a_upper) = self.a.size_hint();
        let (b_lower, b_upper) = self.b.size_hint();

        let lower = cmp::min(a_lower, b_lower);

        let upper = match (a_upper, b_upper) {
            (Some(x), Some(y)) => Some(cmp::min(x, y)),
            (Some(x), None) => Some(x),
            (None, Some(y)) => Some(y),
            (None, None) => None,
        };

        (lower, upper)
    }
}
