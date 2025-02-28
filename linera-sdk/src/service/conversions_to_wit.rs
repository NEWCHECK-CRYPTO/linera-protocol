// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Conversions from types declared in [`linera-sdk`] to types generated by [`wit-bindgen`].

use linera_base::{
    crypto::CryptoHash,
    data_types::BlockHeight,
    identifiers::{ApplicationId, BytecodeId, ChainId, MessageId},
};

use super::wit::service_runtime_api as wit_service_api;

impl From<CryptoHash> for wit_service_api::CryptoHash {
    fn from(hash_value: CryptoHash) -> Self {
        let parts = <[u64; 4]>::from(hash_value);

        wit_service_api::CryptoHash {
            part1: parts[0],
            part2: parts[1],
            part3: parts[2],
            part4: parts[3],
        }
    }
}

impl From<BlockHeight> for wit_service_api::BlockHeight {
    fn from(block_height: BlockHeight) -> Self {
        wit_service_api::BlockHeight {
            inner0: block_height.0,
        }
    }
}

impl From<ChainId> for wit_service_api::ChainId {
    fn from(chain_id: ChainId) -> Self {
        wit_service_api::ChainId {
            inner0: chain_id.0.into(),
        }
    }
}

impl From<BytecodeId> for wit_service_api::BytecodeId {
    fn from(bytecode_id: BytecodeId) -> Self {
        wit_service_api::BytecodeId {
            contract_blob_hash: bytecode_id.contract_blob_hash.into(),
            service_blob_hash: bytecode_id.service_blob_hash.into(),
        }
    }
}

impl From<MessageId> for wit_service_api::MessageId {
    fn from(message_id: MessageId) -> Self {
        wit_service_api::MessageId {
            chain_id: message_id.chain_id.into(),
            height: message_id.height.into(),
            index: message_id.index,
        }
    }
}

impl From<ApplicationId> for wit_service_api::ApplicationId {
    fn from(application_id: ApplicationId) -> Self {
        wit_service_api::ApplicationId {
            bytecode_id: application_id.bytecode_id.into(),
            creation: application_id.creation.into(),
        }
    }
}
