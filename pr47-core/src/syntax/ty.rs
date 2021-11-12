use crate::diag::location::{SourceLoc, SourceRange};
use crate::syntax::id::Identifier;

pub enum ConcreteType<'a> {
    SimpleType(ConcreteSimpleType<'a>),
    GenericType(ConcreteGenericType<'a>)
}

pub struct ConcreteSimpleType<'a> {
    pub content: ConcreteSimpleTypeContent<'a>,
    pub range: SourceRange
}

pub enum ConcreteSimpleTypeContent<'a> {
    VoidType,
    ByteType,
    IntType,
    FloatType,
    CharType,
    StringType,
    DeducedType,
    UserType(Identifier<'a>)
}

pub struct ConcreteGenericType<'a> {
    pub base: Identifier<'a>,
    pub inner: Box<ConcreteType<'a>>,
    pub left_angle: SourceLoc,
    pub right_angle: SourceLoc
}
