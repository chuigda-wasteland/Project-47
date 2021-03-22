use crate::util::mstring::StringHandle;
use crate::util::location::{SingleLineRange, SourceLocation};

#[derive(Debug)]
pub enum CSTType {
    SimpleType(CSTSimpleType),
    GenericType(CSTGenericType)
}

#[derive(Debug)]
pub struct CSTSimpleType {
    pub content: CSTSimpleTypeContent,
    pub range: SingleLineRange
}

#[derive(Debug)]
pub enum CSTSimpleTypeContent {
    VoidType,
    ByteType,
    IntType,
    FloatType,
    CharType,
    StringType,
    DeducedType,
    UserType(StringHandle)
}

#[derive(Debug)]
pub struct CSTGenericType {
    pub inner: Box<CSTType>,
    pub left_angle: SourceLocation,
    pub right_angle: SourceLocation
}
