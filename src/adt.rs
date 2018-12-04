pub extern crate holochain_core_types;
pub extern crate holochain_wasm_utils;
pub use hdk::*;
pub use entries::*;

use std::ptr;

/// The Constructor for a Trie
///
///     - Returns a Result type, which contains either Ok(()) or Err()
///         - Ok(()) if creation of the root node succeeded.
///         - Err(()) if failure for any reason.
///
pub fn Trie(type: str, bucketing: bool) -> Result<JsonString, String>
{
    let root_node: Trie;
    root_node.data = type;
    root_node.bucketing = bucketing;
    match commit_entry(&root_node)
    {
        Ok(address) => Ok(address.into()),
        Err(e) =>
        {
            println!("Error in Trie initialization: {:?}.", e);
            Err(e.into());
        },
    }
}

/// The insert() function for each word.
///
///     - Adds provided string to the trie.
///     - Also adds link from anchor of type `category` to root of string.
///       anchorText is `string` itself, or `id` if provided.
///
pub fn insert(name: Address, data: String, category: Option<str>, id: Option<i32>) -> Result<JsonString, String>
{
    match get_entry(name)
    {
        Ok(t) => {
            match t
            {
                Some(Trie) => { println!("Trie found, proceeding to insert."); },
                None => { return Err("No such Trie exists"); },
            }
        },
        Err(e) => { panic!("Issue with get_entry() with finding root with error: {:?}.", e) },
    }
    let mut levelpeg: i32 = 0;
    let mut traverser: Address = name;

    for i in 0..data.len()
    {
        match get_links(&traverser, data[i])
        {
            Ok(t) => { traverser = t[0]; continue; },
            Err(e) => { levelpeg = i; break; },
        }
    }

    for j in levelpeg..data.len() + 1
    {
        let node = TrieNode {
            data: data[j],
            level: j,
        };
        if j == data.len() + 1 { node.data = "\0"; node.level = -2; }
        match commit_entry(&node)
        {
            Ok(address) => {
                if j == data.len() + 1
                {
                    match link_entries(&traverser, &address, data)
                    {
                        Ok(t) => { return Ok(address); },
                        Err(e) => { return Err("Linking Null failed in insert with error {:?}.", e); },
                    }
                }
                match link_entries(&traverser, &address, data[j])
                {
                    Ok(t) => { traverser = address; continue; },
                    Err(e) => { return Err("Linking nodes failed in insert with error {:?}.", e); },
                }
            },
            Err(e) => { return Err("Committing node failed with error {:?}.", e) },
        }
    }
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
pub fn lookup(data: String) -> Result<bool, String>
{
    for i in 0..data.len()
    {
        match get_links(&traverser, data[i])
        {
            Ok(t) => { traverser = t[0]; continue; },
            Err(e) => { return Err("Failed locating part of the word."); },
        }
    }

    match get_links(&traverser, data)
    {
        Ok(t) => { Ok(true) }.
        Err(e) => { Err("No null terminator found.") }.
    }
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
