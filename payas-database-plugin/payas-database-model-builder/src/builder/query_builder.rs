use payas_core_model::mapped_arena::{MappedArena, SerializableSlabIndex};
use payas_database_model::column_path::ColumnIdPathLink;
use payas_database_model::limit_offset::{
    LimitParameter, LimitParameterType, OffsetParameter, OffsetParameterType,
};
use payas_database_model::operation::{DatabaseQuery, DatabaseQueryParameter, OperationReturnType};
use payas_database_model::predicate::{PredicateParameter, PredicateParameterTypeWithModifier};
use payas_database_model::types::{
    DatabaseCompositeType, DatabaseType, DatabaseTypeKind, DatabaseTypeModifier,
};

use super::naming::ToDatabaseQueryName;

use super::resolved_builder::{ResolvedCompositeType, ResolvedType};
use super::{order_by_type_builder, predicate_builder, system_builder::SystemContextBuilding};

pub fn build_shallow(models: &MappedArena<ResolvedType>, building: &mut SystemContextBuilding) {
    for (_, model) in models.iter() {
        if let ResolvedType::Composite(c) = &model {
            let model_type_id = building.get_id(c.name.as_str()).unwrap();
            let shallow_query = shallow_pk_query(model_type_id, c);
            let collection_query = shallow_collection_query(model_type_id, c);

            building
                .queries
                .add(&shallow_query.name.to_owned(), shallow_query);
            building
                .queries
                .add(&collection_query.name.to_owned(), collection_query);
        }
    }
}

pub fn build_expanded(building: &mut SystemContextBuilding) {
    for (model_type_id, model_type) in building.database_types.iter() {
        if let DatabaseTypeKind::Composite(DatabaseCompositeType { .. }) = &model_type.kind {
            {
                let operation_name = model_type.pk_query();
                let query = expanded_pk_query(model_type_id, model_type, building);
                let existing_id = building.queries.get_id(&operation_name).unwrap();
                building.queries[existing_id] = query;
            }
            {
                let operation_name = model_type.collection_query();
                let query = expanded_collection_query(model_type_id, model_type, building);
                let existing_id = building.queries.get_id(&operation_name).unwrap();
                building.queries[existing_id] = query;
            }
        }
    }
}

fn shallow_pk_query(
    model_type_id: SerializableSlabIndex<DatabaseType>,
    typ: &ResolvedCompositeType,
) -> DatabaseQuery {
    let operation_name = typ.pk_query();
    DatabaseQuery {
        name: operation_name,
        parameter: DatabaseQueryParameter {
            predicate_param: None,
            order_by_param: None,
            limit_param: None,
            offset_param: None,
        },
        return_type: OperationReturnType {
            type_id: model_type_id,
            is_primitive: false,
            type_name: typ.name.clone(),
            type_modifier: DatabaseTypeModifier::NonNull,
        },
    }
}

fn expanded_pk_query(
    model_type_id: SerializableSlabIndex<DatabaseType>,
    model_type: &DatabaseType,
    building: &SystemContextBuilding,
) -> DatabaseQuery {
    let operation_name = model_type.pk_query();
    let existing_query = building.queries.get_by_key(&operation_name).unwrap();

    let pk_param = pk_predicate_param(model_type_id, model_type, building);

    DatabaseQuery {
        name: operation_name,
        parameter: DatabaseQueryParameter {
            predicate_param: Some(pk_param),
            order_by_param: None,
            limit_param: None,
            offset_param: None,
        },
        return_type: existing_query.return_type.clone(),
    }
}

pub fn pk_predicate_param(
    model_type_id: SerializableSlabIndex<DatabaseType>,
    model_type: &DatabaseType,
    building: &SystemContextBuilding,
) -> PredicateParameter {
    let pk_field = model_type.pk_field().unwrap();

    PredicateParameter {
        name: pk_field.name.to_string(),
        type_name: pk_field.typ.type_name().to_string(),
        typ: PredicateParameterTypeWithModifier {
            type_id: building
                .predicate_types
                .get_id(pk_field.typ.type_name())
                .unwrap(),
            type_modifier: DatabaseTypeModifier::NonNull,
        },
        column_path_link: pk_field
            .relation
            .self_column()
            .map(|column_id| ColumnIdPathLink {
                self_column_id: column_id,
                linked_column_id: None,
            }),
        underlying_type_id: model_type_id,
    }
}

fn shallow_collection_query(
    model_type_id: SerializableSlabIndex<DatabaseType>,
    model: &ResolvedCompositeType,
) -> DatabaseQuery {
    let operation_name = model.collection_query();
    DatabaseQuery {
        name: operation_name,
        parameter: DatabaseQueryParameter {
            predicate_param: None,
            order_by_param: None,
            limit_param: None,
            offset_param: None,
        },
        return_type: OperationReturnType {
            type_id: model_type_id,
            type_name: model.name.clone(),
            is_primitive: false,
            type_modifier: DatabaseTypeModifier::List,
        },
    }
}

fn expanded_collection_query(
    model_type_id: SerializableSlabIndex<DatabaseType>,
    model_type: &DatabaseType,
    building: &SystemContextBuilding,
) -> DatabaseQuery {
    let operation_name = model_type.collection_query();
    let existing_query = building.queries.get_by_key(&operation_name).unwrap();

    let predicate_param = collection_predicate_param(model_type_id, model_type, building);
    let order_by_param = order_by_type_builder::new_root_param(&model_type.name, false, building);
    let limit_param = limit_param(building);
    let offset_param = offset_param(building);

    DatabaseQuery {
        name: operation_name.clone(),
        parameter: DatabaseQueryParameter {
            predicate_param: Some(predicate_param),
            order_by_param: Some(order_by_param),
            limit_param: Some(limit_param),
            offset_param: Some(offset_param),
        },
        return_type: existing_query.return_type.clone(),
    }
}

pub fn limit_param(building: &SystemContextBuilding) -> LimitParameter {
    let param_type_name = "Int".to_string();

    LimitParameter {
        name: "limit".to_string(),
        typ: LimitParameterType {
            type_name: param_type_name.clone(),
            type_id: building.get_id(&param_type_name).unwrap(),
            type_modifier: DatabaseTypeModifier::Optional,
        },
    }
}

pub fn offset_param(building: &SystemContextBuilding) -> OffsetParameter {
    let param_type_name = "Int".to_string();

    OffsetParameter {
        name: "offset".to_string(),
        typ: OffsetParameterType {
            type_name: param_type_name.clone(),
            type_id: building.get_id(&param_type_name).unwrap(),
            type_modifier: DatabaseTypeModifier::Optional,
        },
    }
}

pub fn collection_predicate_param(
    model_type_id: SerializableSlabIndex<DatabaseType>,
    model_type: &DatabaseType,
    building: &SystemContextBuilding,
) -> PredicateParameter {
    let param_type_name = predicate_builder::get_parameter_type_name(&model_type.name);
    PredicateParameter {
        name: "where".to_string(),
        type_name: param_type_name.clone(),
        typ: PredicateParameterTypeWithModifier {
            type_id: building.predicate_types.get_id(&param_type_name).unwrap(),
            type_modifier: DatabaseTypeModifier::Optional,
        },
        column_path_link: None,
        underlying_type_id: model_type_id,
    }
}