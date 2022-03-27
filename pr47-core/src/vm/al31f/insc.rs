//! ## `insc.rs`: defines instruction set for the VM.

use std::ptr::NonNull;

use crate::data::generic::{GenericTypeCtor, GenericTypeVT};
use crate::data::tyck::TyckInfo;

/// An VM instruction
///
/// This is a tri-address like instruction set for register machine.
#[cfg_attr(test, derive(Debug), derive(VariantCount))]
pub enum Insc {
    /// `MOV` [SRC] [DEST]
    ///
    /// Move the value in register `SRC` to `DEST`
    Move(usize, usize),

    /// `ADD-INT [INT@SRC1] [INT@SRC2] [DEST]`
    ///
    /// Add integers in register `SRC1` and `SRC2`, put result to register `DEST`,
    /// **No type checking.**
    AddInt(usize, usize, usize),

    /// `ADD-FLOAT [FLOAT@SRC1] [FLOAT@SRC2] [DEST]`
    ///
    /// Add floats in register `SRC1` and `SRC2`, put result to register `DEST`,
    /// **No type checking.**
    AddFloat(usize, usize, usize),

    /// `ADD-ANY [SRC1] [SRC2] [DEST]`
    ///
    /// Load numbers in register `SRC1` and `SRC2`, **check types at run time** and perform
    /// appropriate addition calculation accordingly, and put result to register `DEST`.
    AddAny(usize, usize, usize),

    /// `INCR [INT@POS]`
    ///
    /// Increment the integer stored in register `POS`, in place. **No type checking.**
    IncrInt(usize),

    /// `DECR [INT@POS]`
    ///
    /// Decrement the integer stored in register `POS`, in place. **No type checking.**
    DecrInt(usize),

    /// `SUB-INT [INT@SRC1] [INT@SRC2] [DEST]`
    ///
    /// Subtract integers in register `SRC1` and `SRC2`, put result to register `DEST`,
    /// **No type checking.**
    SubInt(usize, usize, usize),

    /// `SUB-FLOAT [FLOAT@SRC1] [FLOAT@SRC1] [DEST]`
    ///
    /// Subtract floats in register `SRC1` and `SRC2`, put result to register `DEST`,
    /// **No type checking.**
    SubFloat(usize, usize, usize),

    /// `SUB-ANY [SRC1] [SRC2] [DEST]`
    ///
    /// Load numbers in register `SRC1` and `SRC2`, **check types at run time** and perform
    /// appropriate subtraction calculation accordingly, and put result to register `DEST`.
    SubAny(usize, usize, usize),

    /// `MUL-INT [INT@SRC1] [INT@SRC2] [DEST]`
    ///
    /// Multiply integers in register `SRC1` and `SRC2`, put result to register `DEST`,
    /// **No type checking.**
    MulInt(usize, usize, usize),

    /// `MUL-FLOAT [FLOAT@SRC1] [FLOAT@SRC2] [DEST]`
    ///
    /// Multiply floats in register `SRC1` and `SRC2`, put result to register `DEST`,
    /// **No type checking.**
    MulFloat(usize, usize, usize),

    /// `MUL-ANY [SRC1] [SRC2] [DEST]`
    ///
    /// Load numbers in register `SRC1` and `SRC2`, **check types at run time** and perform
    /// appropriate multiplication calculation accordingly, and put result to register `DEST`.
    MulAny(usize, usize, usize),

    /// `DIV-INT [INT@SRC1] [INT@SRC2] [DEST]`
    ///
    /// Divide integer in register `SRC1` by integer in register `SRC2`, put result to register
    /// `DEST`, **No type checking.**
    DivInt(usize, usize, usize),

    /// `DIV-FLOAT [FLOAT@SRC1] [FLOAT@SRC2] [DEST]`
    ///
    /// Divide float in register `SRC1` by float in register `SRC2`, put result to register
    /// `DEST`, **No type checking.**
    DivFloat(usize, usize, usize),

    /// `DIV-ANY [SRC1] [SRC2] [DEST]`
    ///
    /// Load numbers in register `SRC1` and `SRC2`, **check types at run time** and perform
    /// appropriate division calculation accordingly, and put result to register `DEST`.
    DivAny(usize, usize, usize),

    /// `MOD-INT [INT@SRC1] [INT@SRC2] [DEST]`
    ///
    /// Take the remainder of dividing integer in register `SRC1` by integer in register `SRC2`,
    /// put result to register `DEST`, **No type checking.**.
    ModInt(usize, usize, usize),

    /// `MOD-ANY [FLOAT@SRC1] [FLOAT@SRC2] [DEST]`
    ///
    /// **Check data in both `SRC1` and `SRC2` to be integer**, perform integer remainder operation,
    /// and put result to register `DEST`.
    ModAny(usize, usize, usize),

    /// `EQ-VALUE [VALUE@SRC1] [VALUE@SRC2] [DEST]`
    ///
    /// Assume that `SRC1` and `SRC2` are **values of same type**, check their equality. This
    /// instruction should not be used for float comparison. For comparing float values, use
    /// `EQ-FLOAT`.
    EqValue(usize, usize, usize),

    /// `EQ-REF [REF@SRC1] [REF@SRC2] [DEST]`
    ///
    /// Assume that `SRC1` and `SRC2` are both **references**, check their equality.
    EqRef(usize, usize, usize),

    /// `EQ-ANY [SRC1] [SRC2] [DEST]`
    ///
    /// Make no assumptions on `SRC1` and `SRC2`, check their equality.
    EqAny(usize, usize, usize),

    /// `NE-VALUE [VALUE@SRC1] [VALUE@SRC2] [DEST]`
    ///
    /// Similar to `EQ-VALUE` but yields inverted result.
    NeValue(usize, usize, usize),

    /// `NE-REF [REF@SRC1] [REF@SRC2] [DEST]`
    ///
    /// Similar to `EQ-REF` but yields inverted result.
    NeRef(usize, usize, usize),

    /// `NE-ANY [SRC1] [SRC2] [DEST]`
    ///
    /// Similar to `EQ-ANY` but yields inverted result.
    NeAny(usize, usize, usize),

    /// `LT-INT [INT@SRC1] [INT@SRC2] [DEST]`
    ///
    /// Check if integer in register `SRC1` is less than integer in register `SRC2`, put the boolean
    /// result to `DEST`. **No type checking.**
    LtInt(usize, usize, usize),

    /// `LT-FLOAT [SRC1] [SRC2] [DEST]`
    ///
    /// Check if float in register `SRC1` is less than float in register `SRC2`, put the boolean
    /// result to `DEST`. **No type checking.**
    LtFloat(usize, usize, usize),

    /// `LT-ANY [SRC1] [SRC2] [DEST]`
    ///
    /// Load numbers in register `SRC1` and `SRC2`, **check types at run time** and perform
    /// appropriate less-than comparison accordingly, and put result to register `DEST`.
    LtAny(usize, usize, usize),

    /// `GT-INT [INT@SRC1] [INT@SRC2] [DEST]`
    ///
    /// Similar to `LT-INT` but yields inverted result.
    GtInt(usize, usize, usize),

    /// `GT-FLOAT [SRC1] [SRC2] [DEST]`
    ///
    /// Similar to `LT-FLOAT` but yields inverted result.
    GtFloat(usize, usize, usize),

    /// `GT-ANY [SRC1] [SRC2] [DEST]`
    ///
    /// Similar to `LT-ANY` but yields inverted result.
    GtAny(usize, usize, usize),

    /// `LE-INT [INT@SRC1] [INT@SRC2] [DEST]`
    ///
    /// Check if integer in register `SRC1` is less than or equal to integer in register `SRC2`,
    /// put the boolean result to `DEST`. **No type checking.**
    LeInt(usize, usize, usize),

    /// `LE-FLOAT [SRC1] [SRC2] [DEST]`
    ///
    /// Check if float in register `SRC1` is less than or equal to float in register `SRC2`,
    /// put the boolean result to `DEST`. **No type checking.**
    LeFloat(usize, usize, usize),

    /// `LE-ANY [SRC1] [SRC2] [DEST]`
    ///
    /// Load numbers in register `SRC1` and `SRC2`, **check types at run time** and perform
    /// appropriate less-than-or-equal-to comparison accordingly, and put result to register `DEST`.
    LeAny(usize, usize, usize),

    /// `GE-INT [INT@SRC1] [INT@SRC2] [DEST]`
    ///
    /// Similar to `LE-INT` but yields inverted result.
    GeInt(usize, usize, usize),

    /// `GE-FLOAT [SRC1] [SRC2] [DEST]`
    ///
    /// Similar to `LE-FLOAT` but yields inverted result.
    GeFloat(usize, usize, usize),

    /// `GE-ANY [SRC1] [SRC2] [DEST]`
    ///
    /// Similar to `LE-ANY` but yields inverted result.
    GeAny(usize, usize, usize),

    /// `BITAND-INT [INT@SRC1] [INT@SRC2] [DEST]`
    ///
    /// Bit-and integers in register `SRC1` and `SRC2`, put result to register `DEST`.
    /// **No type checking.**
    BAndInt(usize, usize, usize),

    /// `BITAND-ANY [SRC1] [SRC2] [DEST]`
    ///
    /// **Check data in both `SRC1` and `SRC2` to be integer**, perform integer bit-and operation,
    /// and put result to register `DEST`.
    BAndAny(usize, usize, usize),

    /// `BITOR-INT [INT@SRC1] [INT@SRC2] [DEST]`
    ///
    /// Bit-or integers in register `SRC1` and `SRC2`, put result to register `DEST`.
    /// **No type checking.**
    BOrInt(usize, usize, usize),

    /// `BITOR-ANY [SRC1] [SRC2] [DEST]`
    ///
    /// **Check data in both `SRC1` and `SRC2` to be integer**, perform integer bit-or operation,
    /// and put result to register `DEST`.
    BOrAny(usize, usize, usize),

    /// `BITXOR-INT [INT@SRC1] [INT@SRC2] [DEST]`
    ///
    /// Bit-xor integers in register `SRC1` and `SRC2`, put result to register `DEST`.
    /// **No type checking.**
    BXorInt(usize, usize, usize),

    /// `BITXOR-ANY [SRC1] [SRC2] [DEST]`
    ///
    /// **Check data in both `SRC1` and `SRC2` to be integer**, perform integer bit-xor operation,
    /// and put result to register `DEST`.
    BXorAny(usize, usize, usize),

    /// `BITNOT-INT [SRC] [DEST]`
    ///
    /// Bit-not integer in register `SRC`, put the result to register `DEST`.
    /// **No type checking.**
    BNotInt(usize, usize),

    /// `BITNOT-ANY [SRC] [DEST]`
    ///
    /// **Check data in `SRC` to be integer**, perform integer bit-not operation,
    /// and put result to register `DEST`
    BNotAny(usize, usize),

    /// `NEG-INT [SRC] [DEST]`
    ///
    /// Negate the integer in register `SRC`, put the result to register `DEST`,
    /// **No type checking.**
    NegInt(usize, usize),

    /// `NEG-FLOAT [SRC] [DEST]`
    ///
    /// Negate the float in register `SRC`, put the result to register `DEST`.
    /// **No type checking.**
    NegFloat(usize, usize),

    /// `NEG-ANY [SRC] [DEST]`
    ///
    /// **Check data in `SRC` to be integer**, negate the integer and put the result into register
    /// `DEST`.
    NegAny(usize, usize),

    /// `AND-BOOL [SRC1] [SRC2] [DEST]`
    ///
    /// Logic-and booleans in registers `SRC1` and `SRC2`, put result into register `DEST`.
    /// **No type checking.**
    AndBool(usize, usize, usize),

    /// `AND-ANY [SRC1] [SRC2] [DEST]`
    ///
    /// **Check data in both `SRC1` and `SRC2` to be boolean**, perform boolean logic-and operation,
    /// and put result to register `DEST`.
    AndAny(usize, usize, usize),

    /// `OR-BOOL [SRC1] [SRC2] [DEST]`
    ///
    /// Logic-or booleans in registers `SRC1` and `SRC2`, put result into register `DEST`.
    /// **No type checking.**
    OrBool(usize, usize, usize),

    /// `OR-ANY [SRC1] [SRC2] [DEST]`
    ///
    /// **Check data in both `SRC1` and `SRC2` to be boolean**, perform boolean logic-or operation,
    /// and put result to register `DEST`.
    OrAny(usize, usize, usize),

    /// `NOT-BOOL [SRC] [DEST]`
    ///
    /// Logic negate the float in register `SRC`, put the result to register `DEST`.
    /// **No type checking**.
    NotBool(usize, usize),

    /// `NOT-ANY [SRC] [DEST]`
    ///
    /// **Check data in `SRC` to be boolean**, perform boolean logic negate operation, and put
    /// result to register `DEST`.
    NotAny(usize, usize),

    /// `SHL-INT [INT@SRC1] [INT@SRC2] [DEST]`
    ///
    /// Left shift the integer in register `SRC1` with the integer in register `SRC2`, put result to
    /// register `DEST`, **No type checking.**
    ShlInt(usize, usize, usize),

    /// `SHL-ANY [SRC1] [SRC2] [DEST]`
    ///
    /// **Check data in both `SRC1` and `SRC2` to be integer**, perform the left-shift operation,
    /// and put result to register `DEST`.
    ShlAny(usize, usize, usize),

    /// `SHR-INT [INT@SRC1] [INT@SRC2] [DEST]`
    ///
    /// Right shift the integer in register `SRC1` with the integer in register `SRC2`, put result
    /// to register `DEST`, **No type checking.**
    ShrInt(usize, usize, usize),

    /// `SHR-ANY [SRC1] [SRC2] [DEST]`
    ///
    /// **Check data in both `SRC1` and `SRC2` to be integer**, perform the right-shift operation,
    /// and put result to register `DEST`.
    ShrAny(usize, usize, usize),

    /// `MAKE-INT-CONST [INT-LIT] [DEST]`
    ///
    /// Put the integer literal `LIT` to register `DEST`.
    MakeIntConst(i64, usize),

    /// `MAKE-FLOAT-CONST [FLOAT-LIT] [DEST]`
    ///
    /// Put the float literal `LIT` to register `DEST`.
    MakeFloatConst(f64, usize),

    /// `MAKE-CHAR-CONST [CHAR-LIT] [DEST]`
    ///
    /// Put the char literal `LIT` to register `DEST`.
    MakeCharConst(char, usize),

    /// `MAKE-BOOL-CONST [BOOL-LIT] [DEST]`
    ///
    /// Put the boolean literal `LIT` to register `DEST`.
    MakeBoolConst(bool, usize),

    /// `MAKE-NULL [DEST]`
    ///
    /// Put a `null` literal to register `DEST`.
    MakeNull(usize),

    /// `LOAD-CONST [CONST-ID] [DEST]`
    ///
    /// Load constant `CONST-ID` from constant pool, and put it to register `DEST`.
    LoadConst(usize, usize),

    /// `SAVE-CONST [CONST] [CONST-ID]`
    ///
    /// Save the value in register `CONST` to constant pool location `CONST-ID`. Using this
    /// instruction outside the initialization stage is a logical error. Compiler should
    /// not generate codes in such a way.
    SaveConst(usize, usize),

    /// `CAST-FLOAT-INT [FLOAT@SRC] [DEST]`
    ///
    /// Convert the float in `SRC` to integer, put the result to register `DEST`.
    /// **No type checking.**
    CastFloatInt(usize, usize),

    // TODO: Rust forbids case from `char` to `i64`. Should we use this?
    // CastCharInt(usize, usize),

    /// `CAST-BOOL-INT [BOOL@SRC] [DEST]`
    ///
    /// Convert the boolean value in `SRC` to integer, put the result into register `DEST`.
    /// **No type checking.**
    CastBoolInt(usize, usize),

    /// `CAST-ANY-INT [SRC] [DEST]`
    CastAnyInt(usize, usize),

    CastIntFloat(usize, usize),
    CastAnyFloat(usize, usize),
    CastAnyChar(usize, usize),

    CastIntBool(usize, usize),
    CastAnyBool(usize, usize),

    /// `IS-NULL [SRC] [DEST]`
    ///
    /// Check if data stored in `SRC` is `null`, and save the boolean result to `DEST`.
    IsNull(usize, usize),

    /// `NULL-CHECK [SRC]`
    ///
    /// Similar to `IS-NULL`, but throws null pointer exception instead
    NullCheck(usize),

    /// `IS-TYPE` [SRC] [TYCK-INFO] [DEST]
    ///
    /// Check if data stored in `SRC` is of `TYCK-INFO` type, and save the boolean result to `DEST`.
    IsType(usize, NonNull<TyckInfo>, usize),

    /// `TYCK [SRC] [TYCK-INFO]`
    ///
    /// Check if data stored in `SRC` satisfies `TYCK-INFO`, throws type checking exception if not.
    TypeCheck(usize, NonNull<TyckInfo>),

    /// `OWNERSHIP-INFO-CHECK [SRC] [MASK]`
    ///
    /// Check if data stored in `SRC` satisfies given `MASK`, throws RTLC exception if not.
    OwnershipInfoCheck(usize, u8),

    /// `CALL-UNCHECKED [FUNC-ID] [ARGS..] [RETS..]`
    ///
    /// Call the function denoted by `FUNC-ID` with given `ARGS`, store the return values to `RETS`.
    /// **No type checking**.
    Call(usize, &'static [usize], &'static [usize]),

    /// `CALL-PTR [SRC] [ARGS..] [RETS..]`
    ///
    /// Call the function pointer or closure stored in `SRC` with given `ARGS`, store the return
    /// values to `RETS`. **No type checking**.
    CallPtr(usize, &'static [usize], &'static [usize]),

    /// `CALL-OVERLOAD [OVERLOAD-TBL] [ARGS..] [RETS..]`
    CallOverload(usize, &'static [usize], &'static [usize]),

    /// `RETURN-NOTHING`
    ReturnNothing,

    /// `RETURN-ONE [RETURN-VALUE-LOC]`
    ReturnOne(usize),

    /// `RETURN [RETURN-VALUE-LOCS...]`
    Return(&'static [usize]),

    /// `FFI-CALL-RTLC [FFI-FUNC-ID] [ARGS..] [RETS..]`
    FFICallRtlc(usize, &'static [usize], &'static [usize]),

    /// `FFI-CALL [FFI-FUNC-ID] [ARGS..] [RETS..]`
    #[cfg(feature = "optimized-rtlc")]
    FFICall(usize, &'static [usize], &'static [usize]),

    /// `FFI-CALL-ASYNC [FUNC-ID] [ARGS..] [RET]`
    ///
    /// Call the async function denoted by `FUNC-ID` with given `ARGS`, store the returned
    /// promise to `RET`. **No type checking**. Please note that when feature `optimized-rtlc`
    /// is enabled, all async FFI calls have RTLC.
    #[cfg(all(feature = "async", feature = "optimized-rtlc"))]
    FFICallAsync(usize, &'static [usize], usize),

    /// `AWAIT [FUT] [RETS..]`
    ///
    /// Await the given promise, store its results into given destinations.
    #[cfg(feature = "async")]
    Await(usize, &'static [usize]),

    #[cfg(all(feature = "async", feature = "al31f-builtin-ops"))]
    Spawn(usize, &'static [usize]),

    /// `RAISE [EXCEPTION]`
    Raise(usize),

    JumpIfTrue(usize, usize),
    JumpIfFalse(usize, usize),
    Jump(usize),

    CreateContainer(GenericTypeCtor, NonNull<GenericTypeVT>, usize),

    CreateClosure(usize, &'static [usize], NonNull<GenericTypeVT>, usize),

    #[cfg(feature = "al31f-builtin-ops")] CreateString(usize),
    #[cfg(feature = "al31f-builtin-ops")] CreateObject(usize),

    #[cfg(feature = "al31f-builtin-ops")] VecIndex(usize, usize, usize),
    #[cfg(feature = "al31f-builtin-ops")] VecIndexPut(usize, usize, usize),
    #[cfg(feature = "al31f-builtin-ops")] VecPush(usize, usize),
    #[cfg(feature = "al31f-builtin-ops")] VecLen(usize, usize),

    #[cfg(feature = "al31f-builtin-ops")] StrClone(usize, usize),
    #[cfg(feature = "al31f-builtin-ops")] StrConcat(&'static [usize], usize),
    #[cfg(feature = "al31f-builtin-ops")] StrLen(usize, usize),
    #[cfg(feature = "al31f-builtin-ops")] StrEquals(usize, usize, usize),

    #[cfg(feature = "al31f-builtin-ops")] ObjectGet(usize, NonNull<str>, usize),
    #[cfg(feature = "al31f-builtin-ops")] ObjectGetDyn(usize, usize, usize),

    #[cfg(feature = "al31f-builtin-ops")] ObjectPut(usize, NonNull<str>, usize),
    #[cfg(feature = "al31f-builtin-ops")] ObjectPutDyn(usize, usize, usize)
}

impl Insc {
    pub unsafe fn unsafe_to_string(&self) -> String {
        match self {
            Insc::Move(src, dst) => format!("%{} = %{}", dst, src),
            Insc::AddInt(src1, src2, dst) => format!("%{} = add int %{}, %{}", dst, src1, src2),
            Insc::AddFloat(src1, src2, dst) => format!("%{} = add float %{}, %{}", dst, src1, src2),
            Insc::AddAny(src1, src2, dst) =>format!("%{} = add ? %{}, %{}", dst, src1, src2),
            Insc::SubInt(src1, src2, dst) => format!("%{} = sub int %{}, %{}", dst, src1, src2),
            Insc::EqValue(src1, src2, dst) => format!("%{} = eq value %{}, %{}", dst, src1, src2),
            Insc::EqRef(src1, src2, dst) => format!("%{} = eq ref %{}, %{}", dst, src1, src2),
            Insc::EqAny(src1, src2, dst) => format!("%{} = eq ? %{}, %{}", dst, src1, src2),
            Insc::MakeIntConst(int_const, dst) => format!("%{} = int ${}", dst, int_const),
            Insc::LoadConst(const_id, dst) => format!("%{} = load {}", dst, const_id),
            Insc::SaveConst(src, const_id) => format!("store {}, %{}", const_id, src),
            Insc::Call(func_id, args, rets) => {
                let mut result: String = String::from("[");
                for (i, ret) /*: (usize, &usize)*/ in rets.iter().enumerate() {
                    result.push('%');
                    result.push_str(&ret.to_string());
                    if i != rets.len() - 1 {
                        result.push(',');
                        result.push(' ');
                    }
                }
                result.push_str("] = call F.");
                result.push_str(&func_id.to_string());
                result.push(' ');
                for (i, arg) /*: (usize, &usize)*/ in args.iter().enumerate() {
                    result.push('%');
                    result.push_str(&arg.to_string());
                    if i != args.len() - 1 {
                        result.push(',');
                        result.push(' ');
                    }
                }
                result
            },
            Insc::FFICall(ffi_func_id, args, rets) => {
                let mut result: String = String::from("[");
                for (i, ret) /*: (usize, &usize)*/ in rets.iter().enumerate() {
                    result.push('%');
                    result.push_str(&ret.to_string());
                    if i != rets.len() - 1 {
                        result.push(',');
                        result.push(' ');
                    }
                }
                result.push_str("] = ffi-call F.");
                result.push_str(&ffi_func_id.to_string());
                result.push(' ');
                for (i, arg) /*: (usize, &usize)*/ in args.iter().enumerate() {
                    result.push('%');
                    result.push_str(&arg.to_string());
                    if i != args.len() - 1 {
                        result.push(',');
                        result.push(' ');
                    }
                }
                result
            },
            Insc::ReturnNothing => "ret".into(),
            Insc::ReturnOne(ret_value_loc) => format!("ret %{}", ret_value_loc),
            Insc::Return(ret_value_locs) => {
                let mut result: String = String::from("ret ");
                for (i, ret_value_loc) /*: (usize, &usize)*/ in ret_value_locs.iter().enumerate() {
                    result.push('%');
                    result.push_str(&ret_value_loc.to_string());
                    if i != ret_value_locs.len() - 1 {
                        result.push(',');
                        result.push(' ');
                    }
                }
                result
            },
            Insc::Raise(exception_loc) => format!("raise %{}", exception_loc),
            Insc::CreateObject(dest) => format!("%{} = new object", dest),
            Insc::JumpIfTrue(condition, dest) => format!("if %{} goto L.{}", condition, dest),
            Insc::JumpIfFalse(condition, dest) => format!("if not %{} goto L.{}", condition, dest),
            Insc::Jump(dest) => format!("goto L.{}", dest),
            _ => todo!()
        }
    }
}

#[cfg(test)]
#[cfg_attr(miri, ignore)]
#[test] fn count_instructions() {
    eprintln!(" [pr47::vm::al31f::insc] Insc::VARIANT_COUNT = {}", Insc::VARIANT_COUNT);
}
