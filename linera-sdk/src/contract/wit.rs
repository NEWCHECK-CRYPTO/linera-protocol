// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Internal module with code generated by [`wit-bindgen`](https://github.com/jvff/wit-bindgen).

#![allow(missing_docs)]

// Export the contract interface.
wit_bindgen::generate!({
    world: "contract",
    export_macro_name: "export_contract",
    pub_export_macro: true,
});

pub use self::linera::app::{base_runtime_api, contract_runtime_api};
