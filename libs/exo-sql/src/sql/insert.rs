use maybe_owned::MaybeOwned;

use crate::PhysicalTable;

use super::{
    column::{Column, ProxyColumn},
    physical_column::PhysicalColumn,
    transaction::{TransactionContext, TransactionStepId},
    ExpressionBuilder, SQLBuilder, SQLParamContainer,
};

/// An insert operation.
#[derive(Debug)]
pub struct Insert<'a> {
    /// The table to insert into.
    pub table: &'a PhysicalTable,
    /// The columns to insert into such as `(age, name)`
    pub columns: Vec<&'a PhysicalColumn>,
    /// The values to insert such as `(30, "John"), (35, "Jane")`
    pub values_seq: Vec<Vec<MaybeOwned<'a, Column<'a>>>>,
    /// The columns to return.
    pub returning: Vec<MaybeOwned<'a, Column<'a>>>,
}

impl<'a> ExpressionBuilder for Insert<'a> {
    /// Build the insert statement for the form `INSERT INTO <table> (<columns>) VALUES (<values>)
    /// RETURNING <returning-columns>`. The `RETURNING` clause is omitted if the list of columns to
    /// return is empty.
    fn build(&self, builder: &mut SQLBuilder) {
        builder.push_str("INSERT INTO ");
        self.table.build(builder);

        builder.push_str(" (");
        builder.without_fully_qualified_column_names(|builder| {
            builder.push_elems(&self.columns, ", ");
        });

        builder.push_str(") VALUES (");

        builder.push_iter(self.values_seq.iter(), "), (", |builder, values| {
            builder.push_elems(values, ", ");
        });
        builder.push(')');

        if !self.returning.is_empty() {
            builder.push_str(" RETURNING ");
            builder.push_elems(&self.returning, ", ")
        }
    }
}

#[derive(Debug)]
pub struct TemplateInsert<'a> {
    pub table: &'a PhysicalTable,
    pub column_names: Vec<&'a PhysicalColumn>,
    pub column_values_seq: Vec<Vec<ProxyColumn<'a>>>,
    pub returning: Vec<Column<'a>>,
}

impl<'a> TemplateInsert<'a> {
    fn has_template_columns(&self) -> bool {
        self.column_values_seq.iter().any(|column_values| {
            column_values
                .iter()
                .any(|value| matches!(value, ProxyColumn::Template { .. }))
        })
    }

    fn expand_row<'b>(
        column_values_seq: &'b [Vec<ProxyColumn<'b>>],
        row_index: usize,
        transaction_context: &TransactionContext,
    ) -> Vec<Vec<MaybeOwned<'b, Column<'b>>>> {
        column_values_seq
            .iter()
            .map(|row| {
                row.iter()
                    .map(|col| match col {
                        ProxyColumn::Concrete(col) => col.as_ref().into(),
                        ProxyColumn::Template { col_index, step_id } => {
                            MaybeOwned::Owned(Column::Param(SQLParamContainer::new(
                                transaction_context.resolve_value(*step_id, row_index, *col_index),
                            )))
                        }
                    })
                    .collect()
            })
            .collect()
    }

    pub fn resolve(
        &'a self,
        prev_step_id: TransactionStepId,
        transaction_context: &TransactionContext,
    ) -> Option<Insert<'a>> {
        let row_count = transaction_context.row_count(prev_step_id);

        // If there are template columns, but no way to resolve them, this operation need not be performed
        // For example, if we are updating concert_artists while updating concerts, and there are no matching concerts
        // (determined by the where param to updateConcerts), then we don't need to update the concert_artists
        if self.has_template_columns() && row_count == 0 {
            None
        } else {
            let TemplateInsert {
                table,
                column_names,
                column_values_seq,
                returning,
            } = self;

            let resolved_cols = (0..row_count)
                .flat_map(move |row_index| {
                    Self::expand_row(column_values_seq, row_index, transaction_context)
                })
                .collect();

            Some(Insert {
                table,
                columns: column_names.clone(),
                values_seq: resolved_cols,
                returning: returning.iter().map(|ret| ret.into()).collect(),
            })
        }
    }
}