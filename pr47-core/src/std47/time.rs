use std::time::{Duration, Instant};

use xjbutil::void::Void;

use crate::data::traits::StaticBase;
use crate::data::tyck::TyckInfoPool;
use crate::data::Value;
use crate::ffi::{FFIException, Signature};
use crate::ffi::sync_fn::{FunctionBase, VMContext};

impl StaticBase<Instant> for Void {}
impl StaticBase<Duration> for Void {}

pub struct DurationForMillisBind();
