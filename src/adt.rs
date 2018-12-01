pub extern crate holochain_core_types;
pub extern crate holochain_wasm_utils;
pub use api::*;

use std::ptr;

/// The Constructor for a Trie
///
/// Returns a Result type, which contains either Ok(()) or Err()
/// Ok(()) if creation of the root node succeeded.
/// Err() if failure for any reason.
pub fn Trie(type: str, bucketing: bool) -> Result<()>
{
    commit_entry(&rootentry); // work with entry!() macro before this
}

pub fn insert(data: str, category: Option<str>, id: Option<i32>) -> Result<()>
{

}

pub fn drop(data: str) -> Result<()>
{

}

pub fn delete(data: str, category: Option<str>, id: Option<i32>) -> Result<()>
{

}

pub fn lookup(data: str) -> Result<bool>
{

}

pub fn find(category: str, id: i32) -> Result<bool>
{

}
