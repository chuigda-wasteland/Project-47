use xjbutil::void::Void;

use crate::data::traits::StaticBase;

impl StaticBase<std::fmt::Error> for Void {}
impl StaticBase<std::io::Error> for Void {}