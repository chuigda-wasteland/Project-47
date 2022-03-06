
pub type DefineSymbol = String; // todo
pub type LocalSymbol = String; // todo
pub type SymbolRef = String; // todo


#[derive(Debug, Clone, PartialEq)]
pub struct FunctionAttr {

}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionType {

}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeBindAttr {

}

#[derive(Debug, Clone, PartialEq)]
pub struct NamedFun {
    pub attr: FunctionAttr,
    pub name: DefineSymbol,
    pub fun: Fun,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Fun {
    pub ftyp: FunctionType,
    pub body: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LetBinding {
    pub bind: (LocalSymbol, Value, Option<TypeBindAttr>),
    pub body: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Let(LetBinding),
    If(If),
    Cond(Cond),
    Switch(Switch),
    While(While),
    Begin(Begin),
    Store(Store),
    Val(Value),
}

#[derive(Debug, Clone, PartialEq)]
pub struct If(pub Value, pub Box<Expr>, pub Box<Expr>); // cond, then, else

#[derive(Debug, Clone, PartialEq)]
pub struct Cond(pub Vec<(Value, Expr)>, pub Box<Expr>);

#[derive(Debug, Clone, PartialEq)]
pub struct Switch(pub Value, pub Vec<(ConstantValue, Expr)>, pub Box<Expr>);

#[derive(Debug, Clone, PartialEq)]
pub struct While(pub Value, pub Box<Expr>, pub Option<Store>);  // cond, body, accum

#[derive(Debug, Clone, PartialEq)]
pub struct Begin (pub Vec<Expr>);

#[derive(Debug, Clone, PartialEq)]
pub struct Store (pub SymbolRef, pub StoreType, pub Box<Expr>);  // name, value

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum StoreType {
    Volatile,
    Atomic,
}

use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Var(SymbolRef),
    Lit(ConstantValue),
    Call(Call),
    Fun(Arc<Fun>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    pub fun: Box<Value>,
    pub args: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    ConstVal(ConstantValue),
    Fun(Arc<Fun>)
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstantValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Char(char),
    Null,
}