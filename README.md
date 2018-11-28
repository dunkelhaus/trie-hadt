# Radix Tree (Trie) HDK

### Overview

 - This HDK provides a wrapper to abstract the low-levels of using the Holograph data structure proposed by Holochain as a Trie. 
 - Accepts a string, the term itself, and grafts it onto the tree in the prefix-searchable manner. 
 - Categorizes the terms with keys, and a `categoryString` to them, which can be used in a different form of lookup (or a dump, since getting all the words from the trie requires BFS/DFS or some other exhaustive search).
     - This is implemented in the backend with anchors.

### Functions Provided [API]

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