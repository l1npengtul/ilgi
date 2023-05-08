use miette::Diagnostic;
use thiserror::Error;

#[derive(Clone, Debug, Error, Diagnostic)]
pub enum IlgiError {

}

pub type IResult<T> = miette::Result<T>;