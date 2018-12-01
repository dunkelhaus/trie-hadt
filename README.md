# Radix Tree (Trie) HADT

## Overview

`trie-hadt` is a library for Rust-based holochain dApps that intend to use a known Abstract Data Type, in this case, a Trie, as a Zome backend. 
  
 - This HDK provides a wrapper to abstract the low-levels of using the Holograph data structure proposed by Holochain as a Trie. 
 - Accepts a string, the term itself, and grafts it onto the tree in the prefix-searchable manner. 
 - Categorizes the terms with keys, and a `categoryString` to them, which can be used in a different form of lookup (or a dump, since getting all the words from the trie requires BFS/DFS or some other exhaustive search).
     - The categorization is implemented in the backend with anchors.

## Usage
First, [Rust](https://www.rust-lang.org/en-US/install.html) must be installed on your computer.

Being a Rust library, `trie-hadt` can be added as a dependency to any Rust crate. When you generate Rust based Zomes with [holochain-cmd](https://github.com/holochain/holochain-cmd) add the HADT to the zome's `Cargo.toml` in a new field labelled `hadt`, under the line `hdk = { path = "..." }`.

Sample:
```
hadt = { path = "<path to folder where trie-hadt is located>" }

```

### Specification for App Development
 - Macro expansions for the entry type definitions have been provided within the `trie-hadt`. 
 - Add `extern crate hadt` to the top of your `src/lib.rs` within the Zome whose backend you intend to use as a Trie.
 - When defining your zome, in `src/lib.rs`, define it like so:

```
#[macro_use]
extern crate hdk;
#[macro_use]
extern crate hadt;

// --- Other crates/modules you may need ---

define_zome! {
    entries: [
		hadt::entries::defineTrie(),
		hadt::entries::defineTrieNode(),
		hadt::entries::defineNull()
    ]

    genesis: || {
        Ok(())
    }
    
    functions: {
        // ... Your Zome Functions ...
    }

```

## Functions Provided [API]

 - Trie(type, bucketing) [Constructor]
     - type, a string, is needed to semantically categorize an entire trie - if higher order tries are a preference.
     - bucketing, a boolean that says whether collisions of terms should be supported, or it should err instead, and not support duplicates.

 - insert(string, [category], [id])
     - Adds provided string to the trie. 
     - Also adds link from anchor of type `category` to root of string. anchorText is `string` itself, or `id` if provided.

 - delete(string, [category], [id])
     - Removes string from trie.
     - Also destroys links from anchorType - `id` needed if specified id was different from standard lookup (by `string`).

 - lookup(string)
     - Looks for the string in the trie - returns `true` if found and `false` if not.

 - find(category, id)
     - This function instead looks with the `id` within the anchorType `category` to lookup rather than use the trie 
     - Only exists for times when ID is different and the word is being found from a different context, where it itself is not known.
     
## Contribute
 - The `trie-hadt` is an open source project. To contribute/add your own `hadt`, feel free to fork, and if any comments, questions, concerns - contact me via email -> [Suraj Jena](jena.suraj.k@gmail.com).
 - Holochain is also an open source project. To contribute, check out [contributing guidelines](https://github.com/holochain/org/blob/master/CONTRIBUTING.md) for our general practices and protocols on participating in the community.

## Built on (& for)

[![Built for Holochain](https://holochain.org/assets/images/holochain/Holochain_logo.png)](http://holochain.org/)
[![Chat](https://img.shields.io/badge/chat-chat%2eholochain%2enet-blue.svg?style=flat-square)](https://chat.holochain.net)
