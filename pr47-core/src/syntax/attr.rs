use crate::util::mstring::StringHandle;
use crate::util::location::{SingleLineRange, SourceLocation, MultiLineRange};

#[derive(Debug)]
pub struct AttrList {
    pub attributes: Vec<Attribute>,

    pub sharp_loc: SourceLocation,
    pub left_bracket_loc: SourceLocation,
    pub right_bracket_loc: SourceLocation
}

#[derive(Debug)]
pub enum Attribute {
    ValueOnly(ValueOnlyAttr),
    KVPair(KeyValuePairAttr)
}

#[derive(Debug)]
pub struct ValueOnlyAttr {
    pub value: AttrValue
}

#[derive(Debug)]
pub struct KeyValuePairAttr {
    pub key: StringHandle,
    pub value: AttrValue,

    pub key_range: SingleLineRange
}

#[derive(Debug)]
pub enum AttrValue {
    IntValue(AttrIntValue),
    IdentifierValue(AttrIdentifierValue),
    StrValue(AttrStrValue),
    List(AttrListValue)
}

#[derive(Debug)]
pub struct AttrIntValue {
    pub value: i64,
    pub value_range: SingleLineRange
}

#[derive(Debug)]
pub struct AttrIdentifierValue {
    pub value: StringHandle,
    pub value_range: SingleLineRange
}

#[derive(Debug)]
pub struct AttrStrValue {
    pub value: StringHandle,
    pub value_range: MultiLineRange
}

#[derive(Debug)]
pub struct AttrListValue {
    pub value: Vec<Attribute>,

    pub left_paren_loc: SourceLocation,
    pub right_paren_loc: SourceLocation
}
