pub extern crate holochain_core_types;
pub extern crate holochain_wasm_utils;
pub use hdk::api::*;
pub use entries::*;

use std::ptr;

/// The Constructor for a Trie
///
///     - Returns a Result type, which contains either Ok(()) or Err()
///         - Ok(()) if creation of the root node succeeded.
///         - Err(()) if failure for any reason.
///
pub fn Trie(type: str, bucketing: bool) -> Result<()>
{
    let root_node: Trie;
    root_node.data = type;
    root_node.bucketing = bucketing;
    let commit_result = commit_entry(&root_node);
    match commit_result
    {
        Ok(v) => { v },
        Err(e) =>
        {
            println!("Error in Trie initialization: {:?}");
            Err(());
        },
    }
    Ok(());
}

/// The insert() function for each word.
///
///     - Adds provided string to the trie.
///     - Also adds link from anchor of type `category` to root of string.
///       anchorText is `string` itself, or `id` if provided.
///
pub fn insert(data: str, category: Option<str>, id: Option<i32>) -> Result<()>
{

}

/// The drop() function for each word - soft delete.
///
///     - A soft delete, just removes the terminator link that designates
///       the word, but retains the TrieNodes it used in case they are
///       in use for other `string`s, or some other reason.
///
pub fn drop(data: str) -> Result<()>
{

}

/// The delete() function for each word - hard delete.
///
///     - Removes string from trie.
///     - Also destroys links from anchorType
///     - `id` needed if specified id was different from standard
///        lookup (by `string`).
///
pub fn delete(data: str, category: Option<str>, id: Option<i32>) -> Result<()>
{

}

/// The lookup() function for each word - string prefix lookup.
///
///     - Looks for the string in the trie - returns `true` if found
///       and `false` if not.
///
pub fn lookup(data: str) -> Result<bool>
{

}

/// The find() function for each word - manual category lookup.
///
///     - This function instead looks with the `id` within the
///        anchorType `category` to lookup rather than use the trie.
///     - Only exists for times when ID is different and the word is
///       being found from a different context, where it itself is not known.
pub fn find(category: str, id: i32) -> Result<bool>
{

}
