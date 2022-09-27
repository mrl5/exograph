use async_graphql_parser::types::OperationType;

use payas_resolver_core::validation::field::ValidatedField;
use payas_resolver_core::{request_context::RequestContext, QueryResponse};

use crate::graphql::execution::system_context::SystemContext;
use crate::graphql::execution_error::ExecutionError;

use super::operation_resolver::{DatabaseOperationResolver, ServiceOperationResolver};

pub struct DataRootElement<'a> {
    pub operation_type: &'a OperationType,
}

impl<'a> DataRootElement<'a> {
    pub async fn resolve(
        &self,
        field: &'a ValidatedField,
        system_context: &'a SystemContext,
        request_context: &'a RequestContext<'a>,
    ) -> Result<QueryResponse, ExecutionError> {
        let name = &field.name;
        let system = &system_context.system;

        match self.operation_type {
            OperationType::Query => {
                let query = system.database_subsystem.queries.get_by_key(name);

                match query {
                    Some(query) => query.execute(field, system_context, request_context).await,
                    None => {
                        let query = system.service_subsystem.queries.get_by_key(name);
                        match query {
                            Some(query) => {
                                query.execute(field, system_context, request_context).await
                            }
                            None => Err(ExecutionError::Generic(format!("No such query {}", name))),
                        }
                    }
                }
            }
            OperationType::Mutation => {
                let mutation = system.database_subsystem.mutations.get_by_key(name);

                match mutation {
                    Some(mutation) => {
                        mutation
                            .execute(field, system_context, request_context)
                            .await
                    }
                    None => {
                        let mutation = system.service_subsystem.mutations.get_by_key(name);
                        match mutation {
                            Some(mutation) => {
                                mutation
                                    .execute(field, system_context, request_context)
                                    .await
                            }
                            None => Err(ExecutionError::Generic(format!(
                                "No such mutation {}",
                                name
                            ))),
                        }
                    }
                }
            }
            OperationType::Subscription => {
                todo!()
            }
        }
    }
}