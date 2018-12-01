//! Holochain Development Kit (HDK) for a Radix Tree / Trie
//!
//! This HDK provides a wrapper to abstract the low-levels of using the Holograph data
//! structure proposed by Holochain as a Trie.
//! Accepts a string, the term itself, and grafts it onto the tree in the prefix-searchable manner.
//! Categorizes the terms with keys, and a `categoryString` to them, which can be used in a
//! different form of lookup (or a dump, since getting all the words from the trie requires
//! BFS/DFS or some other exhaustive search).
//!
#![feature(try_from)]
#![feature(never_type)]
pub extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;
pub extern crate holochain_core_types;
pub extern crate holochain_wasm_utils;

pub mod api;
pub mod entry_definition;
pub mod error;
pub mod global_fns;
pub mod globals;
pub mod init_globals;
pub mod macros;
pub mod adts

pub use holochain_wasm_utils::api_serialization::validation::*;

pub mod meta;

pub use api::*;
pub use holochain_core_types::validation::*;
pub use adts::*;
