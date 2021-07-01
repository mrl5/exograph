use crate::sql::{column::Column, predicate::Predicate};

use payas_model::model::predicate::*;

use async_graphql_value::Value;

use super::{operation_context::OperationContext, sql_mapper::SQLMapper};

impl<'a> SQLMapper<'a, Predicate<'a>> for PredicateParameter {
    fn map_to_sql(
        &'a self,
        argument_value: &'a Value,
        operation_context: &'a OperationContext<'a>,
    ) -> Predicate<'a> {
        let system = operation_context.query_context.system;
        let parameter_type = &system.predicate_types[self.type_id];

        let argument_value = match argument_value {
            Value::Variable(name) => operation_context.resolve_variable(name.as_str()).unwrap(),
            _ => argument_value,
        };

        match &parameter_type.kind {
            PredicateParameterTypeKind::ImplicitEqual => {
                let (op_key_column, op_value_column) =
                    operands(self, argument_value, operation_context);
                Predicate::Eq(op_key_column, op_value_column)
            }
            PredicateParameterTypeKind::Opeartor(parameters) => {
                parameters.iter().fold(Predicate::True, |acc, parameter| {
                    let arg = operation_context.get_argument_field(argument_value, &parameter.name);
                    let new_predicate = match arg {
                        Some(op_value) => {
                            let (op_key_column, op_value_column) =
                                operands(self, op_value, operation_context);
                            Predicate::from_name(&parameter.name, op_key_column, op_value_column)
                        }
                        None => Predicate::True,
                    };

                    Predicate::And(Box::new(acc), Box::new(new_predicate))
                })
            }
            PredicateParameterTypeKind::Composite(parameters) => {
                parameters.iter().fold(Predicate::True, |acc, parameter| {
                    let arg = operation_context.get_argument_field(argument_value, &parameter.name);
                    let new_predicate = match arg {
                        Some(argument_value_component) => {
                            parameter.map_to_sql(argument_value_component, operation_context)
                        }
                        None => Predicate::True,
                    };

                    Predicate::And(Box::new(acc), Box::new(new_predicate))
                })
            }
        }
    }
}

fn operands<'a>(
    param: &'a PredicateParameter,
    op_value: &'a Value,
    operation_context: &'a OperationContext<'a>,
) -> (&'a Column<'a>, &'a Column<'a>) {
    let system = &operation_context.query_context.system;
    let op_physical_column = &param.column_id.as_ref().unwrap().get_column(system);
    let op_key_column = operation_context.create_column(Column::Physical(op_physical_column));
    let op_value_column = operation_context.literal_column(op_value, op_physical_column);
    (op_key_column, op_value_column)
}