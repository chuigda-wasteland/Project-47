#ifndef PR47_AL31FU_IMPORTS_H
#define PR47_AL31FU_IMPORTS_H

#include <array>
#include <cassert>
#include <cstdint>

namespace pr47 {
namespace al31fu {

struct WidePointer {
  size_t ptr;
  size_t trivia;

  constexpr inline WidePointer(uintptr_t ptr, uintptr_t trivia)
    : ptr(ptr), trivia(trivia) {}

  constexpr inline bool operator==(const WidePointer& other) const {
    return ptr == other.ptr && trivia == other.trivia;
  }
};

constexpr uint8_t VALUE_TYPE_MASK = 0b00'000'001;
constexpr uint8_t VALUE_TYPE_TAG_MASK = 0b00'111'000;

enum class ValueTypeTag : uint8_t {
  Int = 0b00'001'000,
  Float = 0b00'010'000,
  Char = 0b00'011'000,
  Bool = 0b00'100'000,
};

constexpr inline size_t MakeValueTypedDataTag(ValueTypeTag tag) {
  return static_cast<size_t>(tag) | VALUE_TYPE_TAG_MASK;
}

struct ValueTypedData {
  size_t tag;

  union Inner {
    int64_t intValue;
    double floatValue;
    char32_t charValue;
    bool boolValue;
    uint64_t repr;

    constexpr inline explicit Inner(int64_t value) : intValue(value) {}
    constexpr inline explicit Inner(double value) : floatValue(value) {}
    constexpr inline explicit Inner(char32_t value) : charValue(value) {}
    constexpr inline explicit Inner(bool value) : boolValue(value) {}
    constexpr inline explicit Inner(uint64_t value) : repr(value) {}
  } inner;

  constexpr inline explicit ValueTypedData(int64_t int_value)
    : tag(MakeValueTypedDataTag(ValueTypeTag::Int)),
      inner(int_value) {}

  constexpr inline explicit ValueTypedData(double float_value)
    : tag(MakeValueTypedDataTag(ValueTypeTag::Float)),
      inner(float_value) {}

  constexpr inline explicit ValueTypedData(char32_t char_value)
    : tag(MakeValueTypedDataTag(ValueTypeTag::Char)),
      inner(char_value) {}

  constexpr inline explicit ValueTypedData(bool bool_value)
    : tag(MakeValueTypedDataTag(ValueTypeTag::Bool)),
      inner(bool_value) {}

  constexpr inline ValueTypeTag GetTag() const {
    return static_cast<ValueTypeTag>(tag & VALUE_TYPE_TAG_MASK);
  }

  constexpr inline int64_t GetAsInt() const {
    assert(GetTag() == ValueTypeTag::Int);
    return inner.intValue;
  }

  constexpr inline double GetAsFloat() const {
    assert(GetTag() == ValueTypeTag::Float);
    return inner.floatValue;
  }

  constexpr inline char32_t GetAsChar() const {
    assert(GetTag() == ValueTypeTag::Char);
    return inner.charValue;
  }

  constexpr inline bool GetAsBool() const {
    assert(GetTag() == ValueTypeTag::Bool);
    return inner.boolValue;
  }

  constexpr inline uint64_t GetRepr() const {
    return inner.repr;
  }
};

union Value {
  WidePointer widePointer;
  ValueTypedData valueTypedData;

  constexpr inline explicit Value(WidePointer wide_pointer)
    : widePointer(wide_pointer) {}

  constexpr inline explicit Value(ValueTypedData value_typed_data)
    : valueTypedData(value_typed_data) {}
};

extern "C" {

[[noreturn]] void pr47_al31fu_rs_rust_panic();

bool
pr47_al31fu_rs_poll_fut(WidePointer wide_ptr,
                        std::array<Value*, 8> *ret_values);

} // extern "C"

} // namespace al31fu
} // namespace pr47

#endif // PR47_AL31FU_IMPORTS_H
