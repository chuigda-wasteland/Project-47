use proc_macro::TokenStream;

pub enum TypeInfo {
    AsyncCombustor,
    Bool,
    Char,
    Combustor,
    Float,
    Int,
    Opaque { token_stream: TokenStream },
    Option { inner: Box<TypeInfo> },
    Ref { mutable: bool, inner: Box<TypeInfo> },
    Result { ok_type: Box<TypeInfo>, err_type: Box<TypeInfo> },
    VMObject { inner: Box<TypeInfo> },
    VMObjectRef { mutable: bool, inner: Box<TypeInfo> },
    VMVec { inner: Box<TypeInfo> },
    VMVecRef { mutable: bool, inner: Box<TypeInfo> }
}
