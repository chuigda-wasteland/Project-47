use crate::diag::location::{SourceLoc, SourceRange};
use crate::syntax::id::Identifier;

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
    UserType(Identifier)
}

pub struct ConcreteGenericType {
    pub base: Identifier,
    pub inner: Box<ConcreteType>,
    pub left_angle: SourceLoc,
    pub right_angle: SourceLoc
}
