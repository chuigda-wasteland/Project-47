use crate::util::mstring::StringHandle;
use crate::util::location::{SingleLineRange, SourceLocation};

#[derive(Debug)]
pub enum ConcreteType {
    SimpleType(ConcreteSimpleType),
    GenericType(ConcreteGenericType)
}

#[derive(Debug)]
pub struct ConcreteSimpleType {
    pub content: ConcreteSimpleTypeContent,
    pub range: SingleLineRange
}

#[derive(Debug)]
pub enum ConcreteSimpleTypeContent {
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
pub struct ConcreteGenericType {
    pub inner: Box<ConcreteType>,
    pub left_angle: SourceLocation,
    pub right_angle: SourceLocation
}
