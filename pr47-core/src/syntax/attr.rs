use smallvec::SmallVec;

use crate::diag::location::{SourceLoc, SourceRange};

pub struct AttrList {
    pub attributes: SmallVec<[Attribute; 4]>,

    pub sharp_loc: SourceLoc,
    pub exclaim_loc: SourceLoc,
    pub left_bracket_loc: SourceLoc,
    pub right_bracket_loc: SourceLoc
}

pub enum Attribute {
    ValueOnly(ValueOnlyAttr),
    KVPair(KeyValuePairAttr)
}

pub struct ValueOnlyAttr {
    pub value: AttrValue
}

pub struct KeyValuePairAttr {
    pub key: String,
    pub value: AttrValue,

    pub key_range: SourceRange
}

pub enum AttrValue {
    IntValue(AttrIntValue),
    IdentifierValue(AttrIdentifierValue),
    StrValue(AttrStrValue),
    CallAlikeValue(CallAlikeValue)
}

pub struct AttrIntValue {
    pub value: i64,
    pub value_range: SourceRange
}

pub struct AttrIdentifierValue {
    pub value: String,
    pub value_range: SourceRange
}

pub struct AttrStrValue {
    pub value: String,
    pub value_range: SourceRange
}

pub struct CallAlikeValue {
    pub func_alike: String,
    pub value: Vec<Attribute>,

    pub func_alike_range: SourceRange,
    pub left_paren_loc: SourceLoc,
    pub right_paren_loc: SourceLoc
}
