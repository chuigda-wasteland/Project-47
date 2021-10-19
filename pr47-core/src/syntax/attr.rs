use crate::util::location::{SingleLineRange, SourceLoc, MultiLineRange};

pub struct AttrList {
    pub attributes: Vec<Attribute>,

    pub sharp_loc: SourceLoc,
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

    pub key_range: SingleLineRange
}

pub enum AttrValue {
    IntValue(AttrIntValue),
    IdentifierValue(AttrIdentifierValue),
    StrValue(AttrStrValue),
    List(AttrListValue)
}

pub struct AttrIntValue {
    pub value: i64,
    pub value_range: SingleLineRange
}

pub struct AttrIdentifierValue {
    pub value: String,
    pub value_range: SingleLineRange
}

pub struct AttrStrValue {
    pub value: String,
    pub value_range: MultiLineRange
}

pub struct AttrListValue {
    pub value: Vec<Attribute>,

    pub left_paren_loc: SourceLoc,
    pub right_paren_loc: SourceLoc
}
