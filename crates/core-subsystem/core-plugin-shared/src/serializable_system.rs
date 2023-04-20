// Copyright Exograph, Inc. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file at the root of this repository.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

use serde::{Deserialize, Serialize};

use super::{
    error::ModelSerializationError, interception::InterceptionMap,
    system_serializer::SystemSerializer,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct SerializableSystem {
    pub subsystems: Vec<SerializableSubsystem>,
    pub query_interception_map: InterceptionMap,
    pub mutation_interception_map: InterceptionMap,
}

impl SystemSerializer for SerializableSystem {
    type Underlying = Self;

    fn serialize(&self) -> Result<Vec<u8>, ModelSerializationError> {
        bincode::serialize(self).map_err(ModelSerializationError::Serialize)
    }

    fn deserialize_reader(
        reader: impl std::io::Read,
    ) -> Result<Self::Underlying, ModelSerializationError> {
        bincode::deserialize_from(reader).map_err(ModelSerializationError::Deserialize)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SerializableSubsystem {
    pub id: String,
    pub subsystem_index: usize,
    pub serialized_subsystem: Vec<u8>,
}
