use serde::Serialize;
use std::fmt::Debug;
use std::rc::Rc;
use thiserror::Error;

use sqlparser::ast::Ident;

use super::BlendContext;
use crate::data::{Row, Value};
use crate::result::Result;

#[derive(Error, Serialize, Debug, PartialEq)]
pub enum FilterContextError {
    #[error("value not found")]
    ValueNotFound,
}

#[derive(Debug)]
pub struct FilterContext<'a> {
    table_alias: &'a str,
    columns: &'a [Ident],
    row: &'a Row,
    next: Option<Rc<FilterContext<'a>>>,
}

impl<'a> FilterContext<'a> {
    pub fn concat(
        filter_context: Option<Rc<FilterContext<'a>>>,
        blend_context: &'a BlendContext<'a>,
    ) -> Option<Rc<FilterContext<'a>>> {
        let BlendContext {
            table_alias,
            columns,
            row,
            next,
            ..
        } = blend_context;

        let filter_context = match &row {
            Some(row) => {
                let filter_context = FilterContext::new(table_alias, &columns, row, filter_context);

                Some(Rc::new(filter_context))
            }
            None => filter_context,
        };

        match next {
            Some(next) => FilterContext::concat(filter_context, &next),
            None => filter_context,
        }
    }

    pub fn new(
        table_alias: &'a str,
        columns: &'a [Ident],
        row: &'a Row,
        next: Option<Rc<FilterContext<'a>>>,
    ) -> Self {
        Self {
            table_alias,
            columns,
            row,
            next,
        }
    }

    pub fn get_value(&self, target: &str) -> Result<&'a Value> {
        let get_value = || {
            self.columns
                .iter()
                .position(|column| column.value == target)
                .and_then(|index| self.row.get_value(index))
        };

        match get_value() {
            None => match &self.next {
                None => Err(FilterContextError::ValueNotFound.into()),
                Some(context) => context.get_value(target),
            },
            Some(value) => Ok(value),
        }
    }

    pub fn get_alias_value(&self, table_alias: &str, target: &str) -> Result<&'a Value> {
        let get_value = || {
            if self.table_alias != table_alias {
                return None;
            }

            self.columns
                .iter()
                .position(|column| column.value == target)
                .and_then(|index| self.row.get_value(index))
        };

        match get_value() {
            None => match &self.next {
                None => Err(FilterContextError::ValueNotFound.into()),
                Some(context) => context.get_alias_value(table_alias, target),
            },
            Some(value) => Ok(value),
        }
    }
}
