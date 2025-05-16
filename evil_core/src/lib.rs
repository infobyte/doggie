#![no_std]

pub mod bsp;
mod evil_core;
mod machine;
mod menu;

pub use evil_core::{BoardSpecificAttackFn, EvilCore};
pub use menu::EvilMenu;
