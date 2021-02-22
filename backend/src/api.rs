// Copyright 2020 The Exonum Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Cryptocurrency API.

use exonum::{
    blockchain::{BlockProof, IndexProof},
    crypto::{Hash, PublicKey},
    messages::{AnyTx, Verified},
    runtime::CallerAddress as Address,
};
use exonum_merkledb::{proof_map::Raw, ListProof, MapProof, ObjectHash};
use exonum_rust_runtime::api::{self, ServiceApiBuilder, ServiceApiState};

use crate::{schema::{SchemaImpl, SchemaUtils}, model::Model};

/// Describes the query parameters for the `get_model` endpoint.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ModelQuery {
    /// Public key of the queried model.
    pub version: u32,
}

/// Proof of existence for specific model.
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelProof {
    /// Proof of the whole models table.
    pub to_table: MapProof<String, Hash>,
    /// Proof of the specific model in this table.
    pub to_model: MapProof<Address, Model, Raw>,
}

/// Wallet history.
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelHistory {
    /// Proof of the list of transaction hashes.
    pub proof: ListProof<Hash>,
    /// List of above transactions.
    pub transactions: Vec<Verified<AnyTx>>,
}

/// Wallet information.
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Proof of the last block.
    pub block_proof: BlockProof,
    /// Proof of the appropriate model.
    pub model_proof: ModelProof,
    /// History of the appropriate model.
    pub model_history: Option<ModelHistory>,
}

/// Public service API description.
#[derive(Debug, Clone, Copy)]
pub struct PublicApi;

impl PublicApi {
    /// Endpoint for getting a single model.
    pub async fn model_info(
        state: ServiceApiState,
        query: ModelQuery,
    ) -> api::Result<ModelInfo> {
        let IndexProof {
            block_proof,
            index_proof,
            ..
        } = state.data().proof_for_service_index("models").unwrap();

        let model_schema = SchemaImpl::new(state.service_data());
        let address = Address::from_key(SchemaUtils::pubkey_from_version(query.version));
        let to_model = model_schema.public.models.get_proof(address);
        let model_proof = ModelProof {
            to_table: index_proof,
            to_model,
        };
        let model = model_schema.public.models.get(&address);
    
        let model_history = model.map(|_| {
            // `history` is always present for existing models.
            let tempAddress: u32 = 0;
            let history = model_schema.model_history.get(&tempAddress);
            let proof = history.get_range_proof(..);
    
            let transactions = state.data().for_core().transactions();
            let transactions = history
                .iter()
                .map(|tx_hash| transactions.get(&tx_hash).unwrap())
                .collect();
    
            ModelHistory {
                proof,
                transactions,
            }
        });
    
        Ok(ModelInfo {
            block_proof,
            model_proof,
            model_history,
        })
    }

    pub async fn get_model(
        state: ServiceApiState,
        query: ModelQuery,
    ) -> api::Result<Model>{
        let model_schema = SchemaImpl::new(state.service_data());
        let versionHash = Address::from_key(SchemaUtils::pubkey_from_version(query.version));
        let model = model_schema.public.models.get(&versionHash).unwrap();
        let res = Some(model);
        res.ok_or_else(|| api::Error::not_found().title("No model with that version"))
        
    }

    //returns -1 in case of the absence of models
    pub async fn latest_model(
        state: ServiceApiState,
        query: (),
    ) -> api::Result<i32>{
        let model_schema = SchemaImpl::new(state.service_data());
        let versionsNum = model_schema.public.models.keys().count() as i32;
        let latest = versionsNum - 1;
        Ok(latest)
    }

    pub async fn get_trainers_scores(
        state: ServiceApiState,
        query: (),
    ) -> api::Result<String>{
        let model_schema = SchemaImpl::new(state.service_data());
        let scores = model_schema.trainers_scores; //unwrap?
        let mut res : String = "{".to_string(); 
        let mut flag = 0; 
        for v in scores.iter() {
            if (flag > 0){
                res.push_str(",");
            }
            flag = 1;
            res.push_str(r#"""#);
            res.push_str(&v.0.object_hash().to_hex());
            res.push_str(r##"":""##);
            res.push_str(&v.1);
            res.push_str(r#"""#);
        }
        res.push_str("}");
        Ok(res)
        
    }
    
    /// Wires the above endpoint to public scope of the given `ServiceApiBuilder`.
    pub fn wire(builder: &mut ServiceApiBuilder) {
        builder
            .public_scope()
            .endpoint("v1/models/info", Self::model_info)
            .endpoint("v1/models/getmodel", Self::get_model)
            .endpoint("v1/models/trainersscores", Self::get_trainers_scores)
            .endpoint("v1/models/latestmodel", Self::latest_model);
    }
}
