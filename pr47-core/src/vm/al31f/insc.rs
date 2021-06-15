//! ## `insc.rs`: defines instruction set for the VM.

use std::ptr::NonNull;

use crate::data::tyck::TyckInfo;

/// An VM instruction
///
/// This is a tri-address like instruction set for register machine.
pub enum Insc {
    /// `ADD-INT <INT@SRC1> <INT@SRC2> [DEST]`
    ///
    /// Add integers in register `SRC1` and `SRC2`, put result to register `DEST`,
    /// **No type checking.**
    AddInt(usize, usize, usize),

    /// `ADD-FLOAT <FLOAT@SRC1> [SRC2] [DEST]`
    ///
    /// Add floats in register `SRC1` and `SRC2`, put result to register `DEST`,
    /// **No type checking.**
    AddFloat(usize, usize, usize),

    /// `ADD-ANY [SRC1] [SRC2] [DEST]`
    ///
    /// Load numbers in register `SRC1` and `SRC2`, **check types at run time** and perform
    /// appropriate addition calculation accordingly, and put result to register `DEST`.
    AddAny(usize, usize, usize),

    /// `INCR [POS]`
    ///
    /// Increment the integer stored in register `POS`, in place. **No type checking.**
    IncrInt(usize),

    /// `DECR [POS]`
    ///
    /// Decrement the integer stored in register `POS`, in place. **No type checking.**
    DecrInt(usize),

    /// `SUB-INT [SRC1] [SRC2] [DEST]`
    ///
    /// Subtract integers in register `SRC1` and `SRC2`, put result to register `DEST`,
    /// **No type checking.**
    SubInt(usize, usize, usize),

    /// `SUB-FLOAT [SRC1] [SRC2] [DEST]`
    ///
    /// Subtract floats in register `SRC1` and `SRC2`, put result to register `DEST`,
    /// **No type checking.**
    SubFloat(usize, usize, usize),

    /// `SUB-ANY [SRC1] [SRC2] [DEST]`
    ///
    /// Load numbers in register `SRC1` and `SRC2`, **check types at run time** and perform
    /// appropriate subtraction calculation accordingly, and put result to register `DEST`.
    SubAny(usize, usize, usize),

    /// `MUL-INT [SRC1] [SRC2] [DEST]`
    ///
    /// Multiply integers in register `SRC1` and `SRC2`, put result to register `DEST`,
    /// **No type checking.**
    MulInt(usize, usize, usize),

    /// `MUL-FLOAT [SRC1] [SRC2] [DEST]`
    ///
    /// Multiply floats in register `SRC1` and `SRC2`, put result to register `DEST`,
    /// **No type checking.**
    MulFloat(usize, usize, usize),

    /// `MUL-ANY [SRC1] [SRC2] [DEST]`
    ///
    /// Load numbers in register `SRC1` and `SRC2`, **check types at run time** and perform
    /// appropriate multiplication calculation accordingly, and put result to register `DEST`.
    MulAny(usize, usize, usize),

    /// `DIV-INT [SRC1] [SRC2] [DEST]`
    ///
    /// Divide integer in register `SRC1` by integer in register `SRC2`, put result to register
    /// `DEST`, **No type checking.**
    DivInt(usize, usize, usize),

    /// `DIV-FLOAT [SRC1] [SRC2] [DEST]`
    ///
    /// Divide float in register `SRC1` by float in register `SRC2`, put result to register
    /// `DEST`, **No type checking.**
    DivFloat(usize, usize, usize),

    /// `DIV-ANY [SRC1] [SRC2] [DEST]`
    ///
    /// Load numbers in register `SRC1` and `SRC2`, **check types at run time** and perform
    /// appropriate division calculation accordingly, and put result to register `DEST`.
    DivAny(usize, usize, usize),

    /// `MOD-INT [SRC1] [SRC2] [DEST]`
    ///
    /// Take the remainder of dividing integer in register `SRC1` by integer in register `SRC2`,
    /// put result to register `DEST`, **No type checking.**.
    ModInt(usize, usize, usize),

    /// `MOD-ANY [SRC1] [SRC2] [DEST]`
    ///
    /// **Check data in both `SRC1` and `SRC2` to be integer**, perform integer remainder operation,
    /// and put result to register `DEST`.
    ModAny(usize, usize, usize),

    /// `EQ-INT [SRC1] [SRC2] [DEST]`
    ///
    /// Check the equality of integers in registers `SRC1` and `SRC2`, put the boolean result to
    /// `DEST`. **No type checking.**
    EqInt(usize, usize, usize),

    /// `EQ-FLOAT [SRC1] [SRC2] [DEST]`
    ///
    /// Check the equality of floats in registers `SRC1` and `SRC2`, put the boolean result to
    /// `DEST`. **No type checking.**
    EqFloat(usize, usize, usize),

    /// `EQ-CHAR [SRC1] [SRC2] [DEST]`
    ///
    /// Check the equality of chars in registers `SRC1` and `SRC2`, put the boolean result to
    /// `DEST`. **No type checking.**
    EqChar(usize, usize, usize),

    /// `EQ-BOOL [SRC1] [SRC2] [DEST]`
    ///
    /// Check the equality of booleans in registers `SRC1` and `SRC2`, put the boolean result to
    /// `DEST`. **No type checking.**
    EqBool(usize, usize, usize),

    /// `EQ-ANY [SRC1] [SRC2] [DEST]`
    ///
    /// Check the **type equality** of data stored in registers `SRC1` and `SRC2`, perform data
    /// equality check accordingly, and save the boolean result to `DEST`.
    EqAny(usize, usize, usize),

    /// `NE-INT [SRC1] [SRC2] [DEST]`
    ///
    /// Similar to `EQ-INT` but yields inverted result.
    NeInt(usize, usize, usize),

    /// `NE-FLOAT [SRC1] [SRC2] [DEST]`
    ///
    /// Similar to `EQ-FLOAT` but yields inverted result.
    NeFloat(usize, usize, usize),

    /// `NE-CHAR [SRC1] [SRC2] [DEST]`
    ///
    /// Similar to `EQ-CHAR` but yields inverted result.
    NeChar(usize, usize, usize),

    /// `NE-BOOL [SRC1] [SRC2] [DEST]`
    ///
    /// Similar to `EQ-BOOL` but yields inverted result.
    NeBool(usize, usize, usize),

    /// `NE-ANY [SRC1] [SRC2] [DEST]`
    ///
    /// Similar to `EQ-ANY` but yields inverted result.
    NeAny(usize, usize, usize),

    /// `LT-INT [SRC1] [SRC2] [DEST]`
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

    /// `GT-INT [SRC1] [SRC2] [DEST]`
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

    /// `LE-INT [SRC1] [SRC2] [DEST]`
    ///
    /// Check if integer in register `SRC1` is less than or equal to integer in register `SRC2`,
    /// put the boolean result to `DEST`. **No type checking.**
    LeInt(usize, usize, usize),

    /// `LE-INT [SRC1] [SRC2] [DEST]`
    ///
    /// Check if float in register `SRC1` is less than or equal to float in register `SRC2`,
    /// put the boolean result to `DEST`. **No type checking.**
    LeFloat(usize, usize, usize),

    /// `LE-ANY [SRC1] [SRC2] [DEST]`
    ///
    /// Load numbers in register `SRC1` and `SRC2`, **check types at run time** and perform
    /// appropriate less-than-or-equal-to comparison accordingly, and put result to register `DEST`.
    LeAny(usize, usize, usize),

    /// `GE-INT [SRC1] [SRC2] [DEST]`
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

    /// `EQ-EEF [SRC1] [SRC2] [DEST]`
    ///
    /// Check if the reference stored in `SRC1` is the same as `SRC2`, and put result to register
    /// `DEST`.
    EqRef(usize, usize, usize),

    /// `NE-REF [SRC1] [SRC2] [DEST]`
    ///
    /// Similar to `EQ-REF` but yields inverted result.
    NeRef(usize, usize, usize),

    /// `BITAND-INT [SRC1] [SRC2] [DEST]`
    ///
    /// Bit-and integers in register `SRC1` and `SRC2`, put result to register `DEST`.
    /// **No type checking.**
    BAndInt(usize, usize, usize),

    /// `BITAND-ANY [SRC1] [SRC2] [DEST]`
    ///
    /// **Check data in both `SRC1` and `SRC2` to be integer**, perform integer bit-and operation,
    /// and put result to register `DEST`.
    BAndAny(usize, usize, usize),

    /// `BITOR-INT [SRC1] [SRC2] [DEST]`
    ///
    /// Bit-or integers in register `SRC1` and `SRC2`, put result to register `DEST`.
    /// **No type checking.**
    BOrInt(usize, usize, usize),

    /// `BITOR-ANY [SRC1] [SRC2] [DEST]`
    ///
    /// **Check data in both `SRC1` and `SRC2` to be integer**, perform integer bit-or operation,
    /// and put result to register `DEST`.
    BOrAny(usize, usize, usize),

    /// `BITXOR-INT [SRC1] [SRC2] [DEST]`
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

    /// `XOR-BOOL [SRC1] [SRC2] [DEST]`
    ///
    /// Logic-xor booleans in registers `SRC1` and `SRC2`, put result into register `DEST`.
    /// **No type checking.**
    XorBool(usize, usize, usize),

    /// `XOR-ANY [SRC1] [SRC2] [DEST]`
    ///
    /// **Check data in both `SRC1` and `SRC2` to be boolean**, perform boolean logic-xor operation,
    /// and put result to register `DEST`.
    XorAny(usize, usize, usize),

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
    ShlInt(usize, usize, usize),
    ShlAny(usize, usize, usize),
    ShrInt(usize, usize, usize),
    ShrAny(usize, usize, usize),

    /// `MAKE-INT-CONST [LIT] [DEST]`
    ///
    /// Put the integer literal `LIT` to register `DEST`.
    MakeIntConst(i64, usize),

    /// `MAKE-FLOAT-CONST [LIT] [DEST]`
    ///
    /// Put the float literal `LIT` to register `DEST`.
    MakeFloatConst(f64, usize),

    /// `MAKE-CHAR-CONST [LIT] [DEST]`
    ///
    /// Put the char literal `LIT` to register `DEST`.
    MakeCharConst(char, usize),

    /// `MAKE-BOOL-CONST [LIT] [DEST]`
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

    CastFloatInt(usize, usize),
    CastCharInt(usize, usize),
    CastBoolInt(usize, usize),
    CastAnyInt(usize, usize),
    CastIntFloat(usize, usize),
    CastAnyFloat(usize, usize),
    CastIntChar(usize, usize),
    CastAnyChar(usize, usize),

    /// `IS-NULL [SRC] [DEST]`
    ///
    /// Check if data stored in `SRC` is `null`, and save the boolean result to `DEST`.
    IsNull(usize, usize),

    /// `NULL-CHECK [SRC]`
    ///
    /// Similar to `IS-NULL`, but throws null pointer exception instead
    NullCheck(usize),

    /// `TYCK [SRC] [TYCK-INFO]`
    ///
    /// Check if data stored `SRC` satisfies `TYCK-INFO`, throws type checking exception if not.
    TypeCheck(usize, NonNull<TyckInfo>),

    /// `CALL-UNCHECKED [FUNC-ID] [ARGS..] [RETS..]`
    ///
    /// Call the function denoted by `FUNC-ID` with given `ARGS`, store the return values to `RETS`.
    /// **No type checking**.
    Call(usize, Vec<usize>, Vec<usize>),

    /// `CALL-TYCK [FUNC-ID] [ARGS..] [RETS..]`
    ///
    /// Similar to `CALL-TYCK`, but **performs type checking**.
    CallTyck(usize, Vec<usize>, Vec<usize>),

    /// `CALL-PTR [SRC] [ARGS..] [RETS..]`
    ///
    /// Call the function stored in function pointer `SRC` with given `ARGS`, store the return
    /// values to `RETS`. **No type checking**.
    CallPtr(usize, Vec<usize>, Vec<usize>),

    /// `CALL-PTR-TYCK [SRC] [ARGS..] [RETS..]`
    ///
    /// Similar to `CALL-PTR`, but **performs type checking**.
    CallPtrTyck(usize, Vec<usize>, Vec<usize>),

    // TODO consider what to use for overloaded function
    // CallOverload(usize, Vec<usize>, Vec<usize>),

    /// `FFI-CALL-TYCK [FFI-FUNC-ID] [ARGS..] [RETS..]`
    FFICallTyck(usize, Vec<usize>, Vec<usize>),

    /// `FFI-CALL-RTLC [FFI-FUNC-ID] [ARGS..] [RETS..]`
    #[cfg(feature = "optimized-rtlc")]
    FFICallRtlc(usize, Vec<usize>, Vec<usize>),

    /// `FFI-CALL [FFI-FUNC-ID] [ARGS..] [RETS..]`
    FFICall(usize, Vec<usize>, Vec<usize>),

    /// `FFI-CALL-ASYNC-TYCK [FUNC-ID] [ARGS..] [RET]`
    ///
    /// Call the async function denoted by `FUNC-ID` with given `ARGS`, store the returned promise
    /// to `RET`. *Performs type checking**.
    FFICallAsyncTyck(usize, Vec<usize>, usize),

    /// `FFI-CALL-ASYNC [FUNC-ID] [ARGS..] [RET]`
    ///
    /// Call the async function denoted by `FUNC-ID` with given `ARGS`, store the returned
    /// promise to `RET`. **No type checking**. Please note that when feature `optimized-rtlc`
    /// is enabled, all async FFI calls have RTLC.
    #[cfg(all(feature = "async", feature = "optimized-rtlc"))]
    FFICallAsync(usize, Vec<usize>, usize),

    /// `FFI-CALL-ASYNC-UNCHECKED [FUNC-ID] [ARGS..] [RET]`
    ///
    /// Call the async function denoted by `FUNC-ID` with given `ARGS`, store the returned promise
    /// to `RET`. *No type checking and no RTLC*.
    #[cfg(all(feature = "async", feature = "no-rtlc"))]
    FFICallAsyncUnchecked(usize, Vec<usize>, usize),

    /// `AWAIT [FUT] [RETS..]`
    ///
    /// Await the given promise, store its results into given destinations.
    #[cfg(feature = "async")]
    Await(usize, Vec<usize>),

    JumpIfTrue(usize, usize),
    JumpIfFalse(usize, usize),
    Jump(usize)
}
