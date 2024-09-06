#![forbid(unsafe_code)]
// #![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]


// pub mod app;
pub mod nodes_and_types;
pub mod graph_traverser;
pub mod graph;