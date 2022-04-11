#ifndef PR47_AL31FU_INSC_H
#define PR47_AL31FU_INSC_H

#include <cstdint>

namespace pr47 {
namespace al31fu {

enum InscOpCode : std::uint32_t {
    Move = 0,

    // Numeric calculation
    AddInt = 1,
    AddFloat = 2,
    AddAny = 3,
    IncrInt = 4,
    DecrInt = 5,
    SubInt = 6,
    SubFloat = 7,
    SubAny = 8,
    MulInt = 9,
    MulFloat = 10,
    MulAny = 11,
    DivInt = 12,
    DivFloat = 13,
    DivAny = 14,
    ModInt = 15,
    ModAny = 16,

    // Comparison, note that `EqAny` is special
    EqValue = 17,
    EqRef = 18,
    EqAny = 19,
    NeValue = 20,
    NeRef = 21,
    NeAny = 22,
    LtInt = 23,
    LtFloat = 24,
    LtAny = 25,
    GtInt = 26,
    GtFloat = 27,
    GtAny = 28,
    LeInt = 29,
    LeFloat = 30,
    LeAny = 31,
    GeInt = 32,
    GeFloat = 33,
    GeAny = 34,

    // Bitwise operations
    BAndInt = 35,
    BAndAny = 36,
    BOrInt = 37,
    BOrAny = 38,
    BXorInt = 39,
    BXorAny = 40,
    BNotInt = 41,
    BNotAny = 42,

    // Negations
    NegInt = 43,
    NegFloat = 44,
    NegAny = 45,

    // Boolean operations
    AndBool = 46,
    AndAny = 47,
    OrBool = 48,
    OrAny = 49,
    NotBool = 50,
    NotAny = 51,

    // Shift
    ShlInt = 52,
    ShlAny = 53,
    ShrInt = 54,
    ShrAny = 55,

    // Constant loading
    MakeIntConst = 56,
    MakeFloatConst = 57,
    MakeCharConst = 58,
    MakeBoolConst = 59,
    MakeNull = 60,
    LoadConst = 61,
    SaveConst = 62, // special insc, only used when performing global VM init

    // Casts
    CastFloatInt = 63,
    CastBoolInt = 64,
    CastAnyInt = 65,

    CastIntFloat = 66,
    CastAnyFloat = 67,

    CastAnyChar = 68,
    CastIntBool = 69,
    CastAnyBool = 70,

    // Checking
    IsNull = 71,
    NullCheck = 72,
    IsType = 73,
    TypeCheck = 74,
    OwnershipInfoCheck = 75,

    // Function calls and return
    Call = 76,
    CallPtr = 77,
    CallOverload = 78,
    ReturnNothing = 79,
    ReturnOne = 80,
    Return = 81,

    // FFI
    FFICallRtlc = 82,
#ifdef PR47_FEATURE_OPTIMIZED_RTLC
    FFICallUnchecked = 83,
#endif
#ifdef PR47_FEATURE_ASYNC
    FFICallAsync = 84,
    Await = 85,
    Spawn = 86,
#endif

    // Exception control flow
    Raise = 87,

    // Control flow
    JumpIfTrue = 88,
    JumpIfFalse = 89,
    Jump = 90,

    // Builtins
    CreateContainer = 91,
    CreateClosure = 92,
    CreateString = 93,
    CreateObject = 94,
    VecIndex = 95,
    VecIndexPut = 96,
    VecPush = 97,
    VecLen = 98,

    StrClone = 99,
    StrConcat = 100,
    StrLen = 101,
    StrEquals = 102,

    ObjectGet = 103,
    ObjectGetDyn = 104,
    ObjectPut = 105,
    ObjectPutDyn = 106
};

} // namespace al31fu
} // namespace pr47

#endif // PR47_AL31FU_INSC_H
