// README examples require `std`, thus the guard here
#![cfg_attr(
	feature = "std", 
	doc = include_str!("../../README.md")
)]

#![forbid(unsafe_code)]
#![cfg_attr(not(feature = "std"), no_std)]

// for no_std
extern crate alloc;

pub mod diagnostic;
pub mod files;
pub mod term;
