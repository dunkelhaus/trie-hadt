#[macro_use]
extern crate hdk;
#[macro_use]
extern crate hadt;

pub mod dictionary;
// --- Other crates/modules ---

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
        main (Public) {
            make_dict: {
                inputs: |dictName: String, bucketing: bool|,
                outputs: |result: JsonString|,
                handler: dictionary::handle_make_dict
            }

            add_word: {
                inputs: |dictName: String, word: String|,
                outputs: |result: JsonString|,
                handler: blog::handle_add_word
            }

            add_word_with_bucket: {
                inputs: |bucketDictName: String, word: String, bucketData: String|,
                outputs: |result: JsonString|,
                handler: blog::handle_add_word_with_bucket
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
