use std::num::{ParseFloatError, ParseIntError};
use std::str::ParseBoolError;

use xjbutil::void::Void;

use crate::data::traits::StaticBase;

impl StaticBase<ParseIntError> for Void {}
impl StaticBase<ParseFloatError> for Void {}
impl StaticBase<ParseBoolError> for Void {}
