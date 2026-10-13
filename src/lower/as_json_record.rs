//! `render json: <record>` for a table-backed model with no `as_json`: Rails' `serializable_hash`, written down from the columns.

use std::collections::HashMap;

use crate::analyze::ClassInfo;
use crate::dialect::{MethodDef, Model, ModelBodyItem};
use crate::expr::ExprNode;
use crate::ident::ClassId;
use crate::schema::{ColumnType, Table};
use crate::ty::Ty;

use super::as_json_poro::encoding_for;
use super::as_json_shape::{JsonPair, PairValue, ShapeError};
use super::as_json_writer::{writer_method, PairEncoding};

// Not the columns alone: these add to `attribute_names` or change what a column's reader answers.
const ATTRIBUTE_MACROS: &[&str] =
    &["attribute", "serialize", "store", "store_accessor", "encrypts", "composed_of"];

pub(super) fn column_writer(
    model: &Model,
    table: &Table,
    registry: &HashMap<ClassId, ClassInfo>,
) -> Result<MethodDef, ShapeError> {
    if model.body.iter().any(|item| match item {
        ModelBodyItem::Unknown { expr, .. } => matches!(
            &*expr.node,
            ExprNode::Send { recv: None, method, .. } if ATTRIBUTE_MACROS.contains(&method.as_str())
        ),
        _ => false,
    }) {
        return Err("the model declares attributes beyond its columns");
    }
    let info = registry.get(&model.name).ok_or("the model has no analyzed class info")?;
    let mut pairs = Vec::new();
    for column in &table.columns {
        match &column.col_type {
            // `writer_body` routes these through the date seam and `encode_datetime`, which a later pass zones for a non-UTC app.
            ColumnType::Date | ColumnType::DateTime => {}
            // Not written: Rails anchors a time to 2000-01-01, quotes a decimal, nests json, and the schema keeps a reference under its logical name.
            ColumnType::Time
            | ColumnType::Decimal { .. }
            | ColumnType::Json
            | ColumnType::Jsonb
            | ColumnType::Binary
            | ColumnType::Reference { .. } => {
                return Err("a column's JSON form is not the one written here");
            }
            _ => {
                let ty = match info.instance_methods.get(&column.name) {
                    Some(Ty::Fn { ret, .. }) => (**ret).clone(),
                    _ => info
                        .attributes
                        .fields
                        .get(&column.name)
                        .cloned()
                        .ok_or("a column has no typed reader")?,
                };
                if !matches!(encoding_for(&ty)?, PairEncoding::Scalar(_)) {
                    return Err("a column's reader is not a JSON scalar");
                }
            }
        }
        pairs.push(JsonPair { key: column.name.clone(), value: PairValue::Reader(column.name.clone()), cond: None });
    }
    writer_method(&model.name, &pairs, Some(table), &[])
}
