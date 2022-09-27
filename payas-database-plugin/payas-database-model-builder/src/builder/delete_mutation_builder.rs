//! Build mutation input types associatd with deletion (<Type>DeletionInput) and
//! the create mutations (delete<Type>, and delete<Type>s)

use super::naming::ToDatabaseMutationNames;
use super::resolved_builder::{ResolvedCompositeType, ResolvedType};
use super::type_builder::ResolvedTypeEnv;
use payas_core_model::mapped_arena::{MappedArena, SerializableSlabIndex};
use payas_database_model::operation::DatabaseMutationKind;
use payas_database_model::types::{DatabaseType, DatabaseTypeKind};

use crate::builder::query_builder;

use super::mutation_builder::MutationBuilder;
use super::system_builder::SystemContextBuilding;
use super::Builder;

pub struct DeleteMutationBuilder;

impl Builder for DeleteMutationBuilder {
    fn type_names(
        &self,
        _resolved_composite_type: &ResolvedCompositeType,
        _models: &MappedArena<ResolvedType>,
    ) -> Vec<String> {
        // delete mutations don't need any special input type (the type for the PK and the type for filtering suffice)
        vec![]
    }

    /// Expand the mutation input types as well as build the mutation
    fn build_expanded(
        &self,
        _resolved_env: &ResolvedTypeEnv,
        building: &mut SystemContextBuilding,
    ) {
        // Since there are no special input types for deletion, no expansion is needed

        for (_, model_type) in building.database_types.iter() {
            if let DatabaseTypeKind::Composite(_) = &model_type.kind {
                let model_type_id = building
                    .database_types
                    .get_id(model_type.name.as_str())
                    .unwrap();

                for mutation in self.build_mutations(model_type_id, model_type, building) {
                    building.mutations.add(&mutation.name.to_owned(), mutation);
                }
            }
        }
    }
}

impl MutationBuilder for DeleteMutationBuilder {
    fn single_mutation_name(model_type: &DatabaseType) -> String {
        model_type.pk_delete()
    }

    fn single_mutation_kind(
        model_type_id: SerializableSlabIndex<DatabaseType>,
        model_type: &DatabaseType,
        building: &SystemContextBuilding,
    ) -> DatabaseMutationKind {
        DatabaseMutationKind::Delete(query_builder::pk_predicate_param(
            model_type_id,
            model_type,
            building,
        ))
    }

    fn multi_mutation_name(model_type: &DatabaseType) -> String {
        model_type.collection_delete()
    }

    fn multi_mutation_kind(
        model_type_id: SerializableSlabIndex<DatabaseType>,
        model_type: &DatabaseType,
        building: &SystemContextBuilding,
    ) -> DatabaseMutationKind {
        DatabaseMutationKind::Delete(query_builder::collection_predicate_param(
            model_type_id,
            model_type,
            building,
        ))
    }
}