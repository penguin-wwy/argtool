use std::marker::Copy;
use std::clone::Clone;
use std::cmp::{Eq, PartialEq};
use std::error::Error;
use std::result::Result;
use std::fmt::{Formatter, Display, Debug};
use std::fmt;

use ::OptTable;

pub type Res<'a> = Result<OptTable<'a>, Fail>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Fail {
    MissingArgument(String),
    UnknownArgument(String),
    DuplicatedArgument(String),
    UnexpectedArgument(String),
}

impl Error for Fail {
    fn description(&self) -> &str {
        match *self {
            Fail::MissingArgument(_) => "missing argument",
            Fail::UnknownArgument(_) => "unknown argument",
            Fail::DuplicatedArgument(_) => "duplicated argument",
            Fail::UnexpectedArgument(_) => "unexpected argument",
        }
    }
}

impl Display for Fail {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Fail::MissingArgument(ref name) => {
                writeln!(f, "missing argument: {}.", *name)
            }
            Fail::UnknownArgument(ref name) => {
                writeln!(f, "unknown argument: {}.", *name)
            }
            Fail::DuplicatedArgument(ref name) => {
                writeln!(f, "duplicated argument: {}.", *name)
            }
            Fail::UnexpectedArgument(ref name) => {
                writeln!(f, "unexpected argument: {}.", *name)
            }
        }
    }
}