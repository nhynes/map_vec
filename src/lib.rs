//! # map_vec: Map and Set APIs backed by `Vec`s.

#![cfg_attr(not(test), no_std)]
#![cfg_attr(feature = "nightly", feature(drain_filter, try_reserve_kind))]

extern crate alloc;

pub mod map;
pub mod set;

pub use map::Map;
pub use set::Set;

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
struct ReadmeDoctests;
