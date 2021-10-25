use crate::diag::location::{SourceLoc, SourceRange};

pub enum ConcreteType {
    SimpleType(ConcreteSimpleType),
    GenericType(ConcreteGenericType)
}

pub struct ConcreteSimpleType {
    pub content: ConcreteSimpleTypeContent,
    pub range: SourceRange
}

pub enum ConcreteSimpleTypeContent {
    VoidType,
    ByteType,
    IntType,
    FloatType,
    CharType,
    StringType,
    DeducedType,
    UserType(String)
}

pub struct ConcreteGenericType {
    pub inner: Box<ConcreteType>,
    pub left_angle: SourceLoc,
    pub right_angle: SourceLoc
}
