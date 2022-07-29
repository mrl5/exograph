use async_graphql_parser::types::Directive;
use async_trait::async_trait;
use serde_json::Value;

use crate::graphql::execution::resolver::FieldResolver;
use crate::graphql::execution_error::ExecutionError;
use crate::graphql::request_context::RequestContext;
use crate::graphql::{execution::system_context::SystemContext, validation::field::ValidatedField};

#[async_trait]
impl FieldResolver<Value> for Directive {
    async fn resolve_field<'e>(
        &'e self,
        field: &ValidatedField,
        _system_context: &'e SystemContext,
        _request_context: &'e RequestContext<'e>,
    ) -> Result<Value, ExecutionError> {
        match field.name.as_str() {
            "name" => Ok(Value::String(self.name.node.as_str().to_owned())),
            "description" => Ok(Value::Null),
            "isRepeatable" => Ok(Value::Bool(false)), // TODO
            "locations" => Ok(Value::Array(vec![])),  // TODO
            "args" => Ok(Value::Array(vec![])),       // TODO
            "__typename" => Ok(Value::String("__Directive".to_string())),
            field_name => Err(ExecutionError::InvalidField(
                field_name.to_owned(),
                "Directive",
            )),
        }
    }
}