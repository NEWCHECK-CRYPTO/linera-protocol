// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use std::sync::Arc;

use async_graphql::{EmptySubscription, Request, Response, Schema};
use linera_sdk::{
    graphql::GraphQLMutationRoot, linera_base_types::WithServiceAbi, views::View, Service,
    ServiceRuntime,
};
use matching_engine::{Operation, Parameters};

use crate::state::MatchingEngineState;

pub struct MatchingEngineService {
    state: Arc<MatchingEngineState>,
    runtime: Arc<ServiceRuntime<Self>>,
}

linera_sdk::service!(MatchingEngineService);

impl WithServiceAbi for MatchingEngineService {
    type Abi = matching_engine::MatchingEngineAbi;
}

impl Service for MatchingEngineService {
    type Parameters = Parameters;

    async fn new(runtime: ServiceRuntime<Self>) -> Self {
        let state = MatchingEngineState::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load state");
        MatchingEngineService {
            state: Arc::new(state),
            runtime: Arc::new(runtime),
        }
    }

    async fn handle_query(&self, request: Request) -> Response {
        let schema = Schema::build(
            self.state.clone(),
            Operation::mutation_root(self.runtime.clone()),
            EmptySubscription,
        )
        .finish();
        schema.execute(request).await
    }
}
