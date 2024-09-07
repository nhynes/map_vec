#![doc = include_str!("../README.md")]
#![cfg_attr(not(any(test, doc)), no_std)]
#![cfg_attr(feature = "nightly", feature(trusted_len, try_reserve_kind))]
#![cfg_attr(any(docsrs, feature = "nightly"), feature(doc_cfg))]

extern crate alloc;

pub mod map;
pub mod set;

#[doc(inline)]
pub use map::Map;

#[doc(inline)]
pub use set::Set;
