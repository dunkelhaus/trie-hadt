use error::{ZomeApiError, ZomeApiResult};
use globals::*;
use holochain_core_types::{
    cas::content::Address,
    entry::{Entry, SerializedEntry},
    error::{CoreError, HolochainError, RibosomeReturnCode, ZomeApiInternalResult},
};
pub use holochain_wasm_utils::api_serialization::validation::*;
use holochain_wasm_utils::{
    api_serialization::{
        get_entry::GetEntryOptions,
        get_links::{GetLinksArgs, GetLinksResult},
        link_entries::LinkEntriesArgs,
        QueryArgs, QueryResult, ZomeFnCallArgs,
    },
    holochain_core_types::{
        hash::HashString,
        json::{JsonString, RawString},
    },
    memory_allocation::*,
    memory_serialization::*,
};
use serde_json;
use std::{convert::TryInto, os::raw::c_char};

//--------------------------------------------------------------------------------------------------
// ZOME API GLOBAL VARIABLES
//--------------------------------------------------------------------------------------------------

lazy_static! {
  /// The `name` property as taken from the DNA.
  pub static ref DNA_NAME: &'static str = &GLOBALS.dna_name;

  /// The hash of the DNA the Zome is embedded within.
  /// This is often useful as a fixed value that is known by all
  /// participants running the DNA.
  pub static ref DNA_HASH: &'static HashString = &GLOBALS.dna_hash;

  /// The identity string used when the chain was first initialized.
  pub static ref AGENT_ID_STR: &'static str = &GLOBALS.agent_id_str;

  /// The hash of your public key.
  /// This is your node address on the DHT.
  /// It can be used for node-to-node messaging with `send` and `receive` functions.
  pub static ref AGENT_ADDRESS: &'static Address = &GLOBALS.agent_address;

  /// The hash of the first identity entry on your chain (The second entry on your chain).
  /// This is your peer's identity on the DHT.
  pub static ref AGENT_INITIAL_HASH: &'static HashString = &GLOBALS.agent_initial_hash;

  #[doc(hidden)]
  /// The hash of the most recent identity entry that has been committed to your chain.
  /// Starts with the same value as AGENT_INITIAL_HASH.
  /// After a call to `update_agent` it will have the value of the hash of the newly committed identity entry.
  pub static ref AGENT_LATEST_HASH: &'static HashString = &GLOBALS.agent_latest_hash;
}

impl From<DNA_NAME> for JsonString {
    fn from(dna_name: DNA_NAME) -> JsonString {
        JsonString::from(RawString::from(dna_name.to_string()))
    }
}

impl From<DNA_HASH> for JsonString {
    fn from(dna_hash: DNA_HASH) -> JsonString {
        JsonString::from(HashString::from(dna_hash.to_string()))
    }
}

impl From<AGENT_ID_STR> for JsonString {
    fn from(agent_id: AGENT_ID_STR) -> JsonString {
        JsonString::from(RawString::from(agent_id.to_string()))
    }
}

impl From<AGENT_ADDRESS> for JsonString {
    fn from(agent_address: AGENT_ADDRESS) -> JsonString {
        JsonString::from(Address::from(agent_address.to_string()))
    }
}

impl From<AGENT_INITIAL_HASH> for JsonString {
    fn from(agent_initial_hash: AGENT_INITIAL_HASH) -> JsonString {
        JsonString::from(HashString::from(agent_initial_hash.to_string()))
    }
}

impl From<AGENT_LATEST_HASH> for JsonString {
    fn from(agent_latest_hash: AGENT_LATEST_HASH) -> JsonString {
        JsonString::from(HashString::from(agent_latest_hash.to_string()))
    }
}

//--------------------------------------------------------------------------------------------------
// SYSTEM CONSTS
//--------------------------------------------------------------------------------------------------

// HC.Status
// WARNING keep in sync with CRUDStatus
bitflags! {
  pub struct EntryStatus: u8 {
    const LIVE     = 1 << 0;
    const REJECTED = 1 << 1;
    const DELETED  = 1 << 2;
    const MODIFIED = 1 << 3;
  }
}

// HC.GetMask
bitflags! {
  pub struct GetEntryMask: u8 {
    const ENTRY      = 1 << 0;
    const ENTRY_TYPE = 1 << 1;
    const SOURCES    = 1 << 2;
  }
}
// explicit `Default` implementation
impl Default for GetEntryMask {
    fn default() -> GetEntryMask {
        GetEntryMask::ENTRY
    }
}

// TODOs
//// HC.LinkAction
//pub enum LinkAction {
//    Add,
//    Delete,
//}
//
//// HC.PkgReq
//pub enum PkgRequest {
//    Chain,
//    ChainOption,
//    EntryTypes,
//}
//
//// HC.PkgReq.ChainOpt
//pub enum ChainOption {
//    None,
//    Headers,
//    Entries,
//    Full,
//}
//
//// HC.Bridge
//pub enum BridgeSide {
//    From,
//    To,
//}
//
//// HC.SysEntryType
//// WARNING Keep in sync with SystemEntryType in holochain-rust
//enum SystemEntryType {
//    Dna,
//    Agent,
//    Key,
//    Headers,
//    Deletion,
//}
//
//mod bundle_cancel {
//    // HC.BundleCancel.Reason
//    pub enum Reason {
//        UserCancel,
//        Timeout,
//    }
//    // HC.BundleCancel.Response
//    pub enum Response {
//        Ok,
//        Commit,
//    }
//}

// Allowed input for close_bundle()
pub enum BundleOnClose {
    Commit,
    Discard,
}

//--------------------------------------------------------------------------------------------------
// API FUNCTIONS
//--------------------------------------------------------------------------------------------------

/// Prints a string through the stdout of the running service, and also
/// writes that string to the logger in the execution context
/// # Examples
/// ```rust
/// # #[macro_use]
/// # extern crate hdk;
/// # extern crate holochain_core_types;
/// # use holochain_core_types::json::JsonString;
///
/// # fn main() {
/// pub fn handle_some_function(content: String) -> JsonString {
///     // ...
///     hdk::debug("write a message to the logs");
///     // ...
///     "whatever".into()
/// }
///
/// # }
/// ```
pub fn debug<J: TryInto<JsonString>>(msg: J) -> ZomeApiResult<()> {
    let mut mem_stack = unsafe { G_MEM_STACK.unwrap() };

    let allocation_of_input = store_as_json(&mut mem_stack, msg)?;

    unsafe {
        hc_debug(allocation_of_input.encode());
    }

    mem_stack
        .deallocate(allocation_of_input)
        .expect("should be able to deallocate input that has been allocated on memory stack");

    Ok(())
}

/// Call an exposed function from another zome.
/// Arguments for the called function are passed as `JsonString`.
/// Returns the value that's returned by the given function as a json str.
/// # Examples
/// In order to utilize `call`, you must have at least two separate Zomes.
/// Here are two Zome examples, where one performs a `call` into the other.
///
/// This first one, is the one that is called into, with the Zome name `summer`.
/// ```rust
/// # #[macro_use]
/// # extern crate hdk;
/// # extern crate serde;
/// # #[macro_use]
/// # extern crate serde_derive;
/// # #[macro_use]
/// # extern crate serde_json;
/// # use hdk::holochain_core_types::json::JsonString;
///
/// # // Adding empty functions so that the cfg(test) build can link.
/// # #[no_mangle]
/// # pub fn hc_init_globals(_: u32) -> u32 { 0 }
/// # #[no_mangle]
/// # pub fn hc_call(_: u32) -> u32 { 0 }
///
/// # fn main() {
///
/// fn handle_sum(num1: u32, num2: u32) -> JsonString {
///     let sum = num1 + num2;
///     return json!({"sum": format!("{}",sum)}).into();
/// }
///
/// define_zome! {
///     entries: []
///
///     genesis: || {
///         Ok(())
///     }
///
///     functions: {
///         main (Public) {
///             sum: {
///                 inputs: |num1: u32, num2: u32|,
///                 outputs: |sum: JsonString|,
///                 handler: handle_sum
///             }
///         }
///     }
/// }
///
/// # }
/// ```
///
/// This second one, is the one that performs the call into the `summer` Zome.
/// ```rust
/// # #![feature(try_from)]
/// # #[macro_use]
/// # extern crate hdk;
/// # extern crate serde;
/// # #[macro_use]
/// # extern crate serde_derive;
/// # #[macro_use]
/// # extern crate serde_json;
/// # #[macro_use]
/// # extern crate holochain_core_types_derive;
///
/// # use hdk::holochain_core_types::hash::HashString;
/// # use hdk::holochain_core_types::json::JsonString;
/// # use hdk::holochain_core_types::error::HolochainError;
///
/// # // Adding empty functions so that the cfg(test) build can link.
/// # #[no_mangle]
/// # pub fn hc_init_globals(_: u32) -> u32 { 0 }
/// # #[no_mangle]
/// # pub fn hc_commit_entry(_: u32) -> u32 { 0 }
/// # #[no_mangle]
/// # pub fn hc_get_entry(_: u32) -> u32 { 0 }
/// # #[no_mangle]
/// # pub fn hc_entry_address(_: u32) -> u32 { 0 }
/// # #[no_mangle]
/// # pub fn hc_query(_: u32) -> u32 { 0 }
/// # #[no_mangle]
/// # pub fn hc_call(_: u32) -> u32 { 0 }
///
/// # fn main() {
///
/// fn handle_check_sum(num1: u32, num2: u32) -> JsonString {
///     #[derive(Serialize, Deserialize, Debug, DefaultJson)]
///     struct SumInput {
///         num1: u32,
///         num2: u32,
///     };
///     let call_input = SumInput {
///         num1: num1,
///         num2: num2,
///     };
///     let maybe_result = hdk::call(
///         "summer",
///         "main",
///         "sum",
///         call_input.into()
///     );
///     match maybe_result {
///         Ok(result) => result.into(),
///         Err(hdk_error) => hdk_error.into(),
///     }
/// }
///
/// define_zome! {
///     entries: []
///
///     genesis: || {
///         Ok(())
///     }
///
///     functions: {
///         main (Public) {
///             check_sum: {
///                 inputs: |num1: u32, num2: u32|,
///                 outputs: |sum: JsonString|,
///                 handler: handle_check_sum
///             }
///         }
///     }
/// }
///
/// # }
/// ```
pub fn call<S: Into<String>>(
    zome_name: S,
    cap_name: S,
    fn_name: S,
    fn_args: JsonString,
) -> ZomeApiResult<JsonString> {
    let mut mem_stack: SinglePageStack;
    unsafe {
        mem_stack = G_MEM_STACK.unwrap();
    }

    // Put args in struct and serialize into memory
    let allocation_of_input = store_as_json(
        &mut mem_stack,
        ZomeFnCallArgs {
            zome_name: zome_name.into(),
            cap_name: cap_name.into(),
            fn_name: fn_name.into(),
            fn_args: String::from(fn_args),
        },
    )?;

    // Call WASMI-able commit
    let encoded_allocation_of_result: u32;
    unsafe {
        encoded_allocation_of_result = hc_call(allocation_of_input.encode() as u32);
    }
    // Deserialize complex result stored in memory and check for ERROR in encoding
    let result = load_string(encoded_allocation_of_result as u32)?;

    // Free result & input allocations.
    mem_stack
        .deallocate(allocation_of_input)
        .expect("deallocate failed");

    // Done
    Ok(result.into())
}

/// Attempts to commit an entry to your local source chain. The entry
/// will have to pass the defined validation rules for that entry type.
/// If the entry type is defined as public, will also publish the entry to the DHT.
/// Returns either an address of the committed entry as a string, or an error.
/// # Examples
/// ```rust
/// # #![feature(try_from)]
/// # extern crate hdk;
/// # extern crate serde_json;
/// # #[macro_use]
/// # extern crate serde_derive;
/// # extern crate holochain_core_types;
/// # #[macro_use]
/// # extern crate holochain_core_types_derive;
/// # use holochain_core_types::json::JsonString;
/// # use holochain_core_types::error::HolochainError;
/// # use holochain_core_types::entry::entry_type::EntryType;
/// # use holochain_core_types::entry::Entry;
/// # fn main() {
///
/// #[derive(Serialize, Deserialize, Debug, DefaultJson)]
/// pub struct Post {
///     content: String,
///     date_created: String,
/// }
///
/// pub fn handle_create_post(content: String) -> JsonString {
///
///     let post_entry = Entry::new(EntryType::App("post".into()), Post{
///         content,
///         date_created: "now".into(),
///     });
///
///     match hdk::commit_entry(&post_entry) {
///         Ok(address) => address.into(),
///         Err(e) => e.into(),
///     }
///
/// }
///
/// # }
/// ```
pub fn commit_entry(entry: &Entry) -> ZomeApiResult<Address> {
    let mut mem_stack: SinglePageStack;
    unsafe {
        mem_stack = G_MEM_STACK.unwrap();
    }

    let allocation_of_input = store_as_json(&mut mem_stack, entry.serialize())?;

    // Call Ribosome's commit_entry()
    let encoded_allocation_of_result: u32;
    unsafe {
        encoded_allocation_of_result = hc_commit_entry(allocation_of_input.encode() as u32);
    }

    // Deserialize complex result stored in wasm memory
    let result: ZomeApiInternalResult = load_json(encoded_allocation_of_result as u32)?;
    // Free result & input allocations
    mem_stack
        .deallocate(allocation_of_input)
        .expect("deallocate failed");
    // Done
    if result.ok {
        Ok(JsonString::from(result.value).try_into()?)
    } else {
        Err(ZomeApiError::from(result.error))
    }
}

/// Retrieves an entry from the local chain or the DHT, by looking it up using
/// its address.
/// # Examples
/// ```rust
/// # extern crate hdk;
/// # extern crate holochain_core_types;
/// # use holochain_core_types::json::JsonString;
/// # use holochain_core_types::cas::content::Address;
/// # fn main() {
/// pub fn handle_get_post(post_address: Address) -> JsonString {
///     // get_entry returns a Result<Option<T>, ZomeApiError>
///     // where T is the type that you used to commit the entry, in this case a Blog
///     // It's a ZomeApiError if something went wrong (i.e. wrong type in deserialization)
///     // Otherwise its a Some(T) or a None
///     match hdk::get_entry(post_address) {
///         Ok(maybe_post) => maybe_post.and_then(|entry| Some(entry.serialize())).into(),
///         Err(e) => e.into(),
///     }
/// }
/// # }
/// ```
pub fn get_entry(address: Address) -> ZomeApiResult<Option<Entry>> {
    Ok(get_entry_result(address, GetEntryOptions {})?
        .and_then(|serialized_entry| Some(serialized_entry.deserialize())))
}

/// Retrieves an entry and meta data from the local chain or the DHT, by looking it up using
/// its address, and a the full options to specify exactly what data to return
pub fn get_entry_result(
    address: Address,
    _options: GetEntryOptions,
) -> ZomeApiResult<Option<SerializedEntry>> {
    let mut mem_stack: SinglePageStack;
    unsafe {
        mem_stack = G_MEM_STACK.unwrap();
    }
    // Put args in struct and serialize into memory
    let allocation_of_input = store_as_json(&mut mem_stack, address)?;

    // Call WASMI-able get_entry
    let encoded_allocation_of_result: u32;
    unsafe {
        encoded_allocation_of_result = hc_get_entry(allocation_of_input.encode() as u32);
    }
    // Deserialize complex result stored in memory
    let result: ZomeApiInternalResult = load_json(encoded_allocation_of_result as u32)?;
    // Free result & input allocations
    mem_stack
        .deallocate(allocation_of_input)
        .expect("deallocate failed");
    // Done
    if result.ok {
        Ok(JsonString::from(result.value).try_into()?)
    } else {
        Err(ZomeApiError::from(result.error))
    }
}

/// Consumes three values, two of which are the addresses of entries, and one of which is a string that defines a
/// relationship between them, called a `tag`. Later, lists of entries can be looked up by using [get_links](fn.get_links.html). Entries
/// can only be looked up in the direction from the `base`, which is the first argument, to the `target`.
/// # Examples
/// ```rust
/// # #![feature(try_from)]
/// # extern crate hdk;
/// # extern crate serde_json;
/// # #[macro_use]
/// # extern crate serde_derive;
/// # extern crate holochain_core_types;
/// # #[macro_use]
/// # extern crate holochain_core_types_derive;
/// # use holochain_core_types::json::JsonString;
/// # use holochain_core_types::error::HolochainError;
/// # use holochain_core_types::entry::entry_type::EntryType;
/// # use holochain_core_types::entry::Entry;
/// # use holochain_core_types::cas::content::Address;
/// # use hdk::AGENT_ADDRESS;
/// # fn main() {
///
/// #[derive(Serialize, Deserialize, Debug, DefaultJson)]
/// pub struct Post {
///     content: String,
///     date_created: String,
/// }
///
/// pub fn handle_link_entries(content: String) -> JsonString {
///     let post_entry = Entry::new(EntryType::App("post".into()), Post{
///         content,
///         date_created: "now".into(),
///     });
///
///     match hdk::commit_entry(&post_entry) {
///         Ok(post_address) => {
///              match hdk::link_entries(
///                 &AGENT_ADDRESS,
///                 &post_address,
///                 "authored_posts"
///             ) {
///                 Ok(link_address) => post_address.into(),
///                 Err(e) => e.into(),
///             }
///         }
///         Err(hdk_error) => hdk_error.into(),
///     }
/// }
/// # }
/// ```
pub fn link_entries<S: Into<String>>(
    base: &Address,
    target: &Address,
    tag: S,
) -> Result<(), ZomeApiError> {
    let mut mem_stack = unsafe { G_MEM_STACK.unwrap() };

    // Put args in struct and serialize into memory
    let allocation_of_input = store_as_json(
        &mut mem_stack,
        LinkEntriesArgs {
            base: base.clone(),
            target: target.clone(),
            tag: tag.into(),
        },
    )?;

    let encoded_allocation_of_result: u32 =
        unsafe { hc_link_entries(allocation_of_input.encode() as u32) };

    // Deserialize complex result stored in memory and check for ERROR in encoding
    let result: ZomeApiInternalResult = load_json(encoded_allocation_of_result as u32)?;
    // Free result & input allocations
    mem_stack
        .deallocate(allocation_of_input)
        .expect("deallocate failed");
    // Done
    if result.ok {
        Ok(JsonString::from(result.value).try_into()?)
    } else {
        Err(ZomeApiError::from(result.error))
    }
}

/// Not Yet Available
// Returns a DNA property, which are defined by the DNA developer.
// They are custom values that are defined in the DNA file
// that can be used in the zome code for defining configurable behaviors.
// (e.g. Name, Language, Description, Author, etc.).
pub fn property<S: Into<String>>(_name: S) -> ZomeApiResult<String> {
    Err(ZomeApiError::FunctionNotImplemented)
}

/// Reconstructs an address of the given entry data.
/// This is the same value that would be returned if `entry_type_name` and `entry_value` were passed
/// to the [commit_entry](fn.commit_entry.html) function and by which it would be retrievable from the DHT using [get_entry](fn.get_entry.html).
/// This is often used to reconstruct an address of a `base` argument when calling [get_links](fn.get_links.html).
/// # Examples
/// ```rust
/// # #![feature(try_from)]
/// # extern crate hdk;
/// # extern crate serde_json;
/// # #[macro_use]
/// # extern crate serde_derive;
/// # extern crate holochain_core_types;
/// # #[macro_use]
/// # extern crate holochain_core_types_derive;
/// # use holochain_core_types::json::JsonString;
/// # use holochain_core_types::error::HolochainError;
/// # use holochain_core_types::entry::entry_type::EntryType;
/// # use holochain_core_types::entry::Entry;
/// # fn main() {
///
/// #[derive(Serialize, Deserialize, Debug, DefaultJson)]
/// pub struct Post {
///     content: String,
///     date_created: String,
/// }
///
/// fn handle_post_address(content: String) -> JsonString {
///
///     let post_entry = Entry::new(EntryType::App("post".into()), Post {
///         content,
///         date_created: "now".into(),
///     });
///
///     match hdk::entry_address(&post_entry) {
///         Ok(address) => address.into(),
///         Err(hdk_error) => hdk_error.into(),
///     }
///
/// }
///
/// # }
/// ```
pub fn entry_address(entry: &Entry) -> ZomeApiResult<Address> {
    let mut mem_stack: SinglePageStack;
    unsafe {
        mem_stack = G_MEM_STACK.unwrap();
    }
    // Put args in struct and serialize into memory
    let allocation_of_input = store_as_json(&mut mem_stack, entry.serialize())?;

    let encoded_allocation_of_result: u32;
    unsafe {
        encoded_allocation_of_result = hc_entry_address(allocation_of_input.encode() as u32);
    }

    // Deserialize complex result stored in memory and check for ERROR in encoding
    let result: ZomeApiInternalResult = load_json(encoded_allocation_of_result as u32)?;
    // Free result & input allocations
    mem_stack
        .deallocate(allocation_of_input)
        .expect("deallocate failed");
    // Done
    if result.ok {
        Ok(JsonString::from(result.value).try_into()?)
    } else {
        Err(ZomeApiError::from(result.error))
    }
}

/// Not Yet Available
pub fn sign<S: Into<String>>(_doc: S) -> ZomeApiResult<String> {
    Err(ZomeApiError::FunctionNotImplemented)
}

/// Not Yet Available
pub fn verify_signature<S: Into<String>>(
    _signature: S,
    _data: S,
    _pub_key: S,
) -> ZomeApiResult<bool> {
    Err(ZomeApiError::FunctionNotImplemented)
}

/// Not Yet Available
pub fn update_entry<S: Into<String>>(
    _entry_type: S,
    _entry: Entry,
    _replaces: Address,
) -> ZomeApiResult<Address> {
    // FIXME
    Err(ZomeApiError::FunctionNotImplemented)
}

/// Not Yet Available
pub fn update_agent() -> ZomeApiResult<Address> {
    Err(ZomeApiError::FunctionNotImplemented)
}

/// Not Yet Available
pub fn remove_entry<S: Into<String>>(_entry: Address, _message: S) -> ZomeApiResult<Address> {
    Err(ZomeApiError::FunctionNotImplemented)
}

/// Consumes two values, the first of which is the address of an entry, `base`, and the second of which is a string, `tag`,
/// used to describe the relationship between the `base` and other entries you wish to lookup. Returns a list of addresses of other
/// entries which matched as being linked by the given `tag`. Links are created in the first place using the Zome API function [link_entries](fn.link_entries.html).
/// Once you have the addresses, there is a good likelihood that you will wish to call [get_entry](fn.get_entry.html) for each of them.
/// # Examples
/// ```rust
/// # extern crate hdk;
/// # extern crate holochain_core_types;
/// # use holochain_core_types::json::JsonString;
/// # use holochain_core_types::cas::content::Address;
///
/// # fn main() {
/// pub fn handle_posts_by_agent(agent: Address) -> JsonString {
///     match hdk::get_links(&agent, "authored_posts") {
///         Ok(result) => result.into(),
///         Err(hdk_error) => hdk_error.into(),
///     }
/// }
/// # }
/// ```
pub fn get_links<S: Into<String>>(base: &Address, tag: S) -> ZomeApiResult<GetLinksResult> {
    let mut mem_stack = unsafe { G_MEM_STACK.unwrap() };
    // Put args in struct and serialize into memory

    let allocation_of_input = store_as_json(
        &mut mem_stack,
        GetLinksArgs {
            entry_address: base.clone(),
            tag: tag.into(),
        },
    )?;

    // Call Ribosome
    let encoded_allocation_of_result: u32 =
        unsafe { hc_get_links(allocation_of_input.encode() as u32) };

    // Deserialize complex result stored in memory
    let result: ZomeApiInternalResult = load_json(encoded_allocation_of_result as u32)?;

    // Free result & input allocations
    mem_stack
        .deallocate(allocation_of_input)
        .expect("deallocate failed");

    if result.ok {
        Ok(JsonString::from(result.value).try_into()?)
    } else {
        Err(ZomeApiError::from(result.error))
    }
}

/// Returns a list of entries from your local source chain, that match a given type.
/// entry_type_name: Specify type of entry to retrieve
/// limit: Max number of entries to retrieve
pub fn query(entry_type_name: &str, start: u32, limit: u32) -> ZomeApiResult<QueryResult> {
    let mut mem_stack: SinglePageStack = unsafe { G_MEM_STACK.unwrap() };

    // Put args in struct and serialize into memory
    let allocation_of_input = store_as_json(
        &mut mem_stack,
        QueryArgs {
            entry_type_name: entry_type_name.to_string(),
            start,
            limit,
        },
    )?;

    let encoded_allocation_of_result: u32 =
        unsafe { hc_query(allocation_of_input.encode() as u32) };

    // Deserialize complex result stored in memory
    let result: ZomeApiInternalResult = load_json(encoded_allocation_of_result as u32)?;
    // Free result & input allocations
    mem_stack
        .deallocate(allocation_of_input)
        .expect("deallocate failed");
    // Done
    if result.ok {
        Ok(JsonString::from(result.value).try_into()?)
    } else {
        Err(ZomeApiError::from(result.error))
    }
}

/// Not Yet Available
pub fn send(_to: Address, _message: serde_json::Value) -> ZomeApiResult<serde_json::Value> {
    Err(ZomeApiError::FunctionNotImplemented)
}

/// Not Yet Available
pub fn start_bundle(_timeout: usize, _user_param: serde_json::Value) -> ZomeApiResult<()> {
    Err(ZomeApiError::FunctionNotImplemented)
}

/// Not Yet Available
pub fn close_bundle(_action: BundleOnClose) -> ZomeApiResult<()> {
    Err(ZomeApiError::FunctionNotImplemented)
}

//--------------------------------------------------------------------------------------------------
// Helpers
//--------------------------------------------------------------------------------------------------

pub fn check_for_ribosome_error(encoded_allocation: u32) -> Result<(), ZomeApiError> {
    // Check for error from Ribosome
    let rib_result = decode_encoded_allocation(encoded_allocation);
    match rib_result {
        // Expecting a 'Success' return code
        Err(ret_code) => match ret_code {
            RibosomeReturnCode::Success => Ok(()),
            RibosomeReturnCode::Failure(err_code) => {
                Err(ZomeApiError::Internal(err_code.to_string()))
            }
        },
        // If we have an allocation, than it should be a CoreError
        Ok(allocation) => {
            let maybe_err: Result<CoreError, HolochainError> =
                load_json_from_raw(allocation.offset() as *mut c_char);
            match maybe_err {
                Err(hc_err) => Err(ZomeApiError::Internal(hc_err.to_string())),
                Ok(core_err) => Err(ZomeApiError::Internal(core_err.to_string())),
            }
        }
    }
}
