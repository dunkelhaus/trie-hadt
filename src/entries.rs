use hdk::holochain_core_types::{
    dna::zome::entry_types::Sharing,
    error::HolochainError,
    json::JsonString,
};
use boolinator::*;
use hdk::{
    self,
    entry_definition::ValidatingEntryType,
};
use serde_json;

pub struct Trie
{
    data: String,
    bucketing: bool,
}

pub struct TrieNode
{
    data: String,
    level: i32,
}

pub struct NullTerm<D>
{
    data: D,
}

pub struct Null
{
    data: "\0",
}

pub fn defineTrie() -> ValidatingEntryType
{
    entry!(
        name: "Trie",
        description: "The root node entry",
        sharing: Sharing::Public,
        native_type: Trie,

        validation_package: || {
            hdk::ValidationPackageDefinition::ChainFull
        },

        validation: |post: Post, _ctx: hdk::ValidationData| {
            (post.content.len() < 280)
                .ok_or_else(|| String::from("Content too long"))
        }
    )
}

pub fn defineTrieNode() -> ValidatingEntryType
{
    entry!(
        name: "TrieNode",
        description: "The trie node entry",
        sharing: Sharing::Public,
        native_type: TrieNode,

        validation_package: || {
            hdk::ValidationPackageDefinition::ChainFull
        },

        validation: |post: Post, _ctx: hdk::ValidationData| {
            (post.content.len() < 280)
                .ok_or_else(|| String::from("Content too long"))
        }
    )
}

pub fn defineNull(bucketing: bool) -> ValidatingEntryType
{
    match (bucketing)
    {
        true => entry!(
            name: "NullTerm",
            description: "The null terminator entry",
            sharing: Sharing::Public,
            native_type: NullTerm,

            validation_package: || {
                hdk::ValidationPackageDefinition::ChainFull
            },

            validation: |post: Post, _ctx: hdk::ValidationData| {
                (post.content.len() < 280)
                    .ok_or_else(|| String::from("Content too long"))
            }
        ),
        false => entry!(
            name: "Null",
            description: "The lone-null entry",
            sharing: Sharing::Public,
            native_type: Null,

            validation_package: || {
                hdk::ValidationPackageDefinition::ChainFull
            },

            validation: |post: Post, _ctx: hdk::ValidationData| {
                (post.content.len() < 280)
                    .ok_or_else(|| String::from("Content too long"))
            }
        )
    }
}
