//! # map_vec: Map and Set APIs backed by Vecs.
#![cfg_attr(not(test), no_std)]
#![cfg_attr(feature = "nightly", feature(drain_filter, shrink_to, try_reserve))]

extern crate alloc;

pub mod map;
pub mod set;

pub use map::Map;
pub use set::Set;
