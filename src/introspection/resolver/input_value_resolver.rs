use graphql_parser::schema::InputValue;
use serde_json::Value;

use crate::introspection::query_context;

use super::resolver::*;
use query_context::QueryContext;

impl<'a> FieldResolver for InputValue<'a, String> {
    fn resolve_field(
        &self,
        query_context: &QueryContext<'_>,
        field: &graphql_parser::query::Field<'_, String>,
    ) -> Value {
        match field.name.as_str() {
            "name" => Value::String(self.name.to_owned()),
            "description" => self
                .description
                .clone()
                .map(|v| Value::String(v))
                .unwrap_or(Value::Null),
            "type" => self
                .value_type
                .resolve_value(query_context, &field.selection_set),
            "defaultValue" => Value::Null, // TODO
            field_name => todo!("Invalid field {:?} for InputValue", field_name), // TODO: Make it a proper error
        }
    }
}
