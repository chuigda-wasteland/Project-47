#ifndef PR47_AL31FU_IMPORTS_H
#define PR47_AL31FU_IMPORTS_H

#include <array>
#include <cassert>
#include <cstdint>

namespace pr47::al31fu {

struct WidePointer {
  size_t ptr;
  size_t trivia;

  constexpr inline WidePointer(uintptr_t ptr, uintptr_t trivia) noexcept
    : ptr(ptr), trivia(trivia) {}

  constexpr inline bool operator==(const WidePointer& other) const noexcept {
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

constexpr inline size_t MakeValueTypedDataTag(ValueTypeTag tag) noexcept {
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

    constexpr inline explicit Inner(int64_t value) noexcept
      : intValue(value) {}
    constexpr inline explicit Inner(double value) noexcept
      : floatValue(value) {}
    constexpr inline explicit Inner(char32_t value) noexcept
      : charValue(value) {}
    constexpr inline explicit Inner(bool value) noexcept
      : boolValue(value) {}
    constexpr inline explicit Inner(uint64_t value) noexcept
      : repr(value) {}
  } inner;

  constexpr inline explicit ValueTypedData(int64_t intValue) noexcept
    : tag(MakeValueTypedDataTag(ValueTypeTag::Int)),
      inner(intValue) {}

  constexpr inline explicit ValueTypedData(double floatValue) noexcept
    : tag(MakeValueTypedDataTag(ValueTypeTag::Float)),
      inner(floatValue) {}

  constexpr inline explicit ValueTypedData(char32_t charValue) noexcept
    : tag(MakeValueTypedDataTag(ValueTypeTag::Char)),
      inner(charValue) {}

  constexpr inline explicit ValueTypedData(bool boolValue) noexcept
    : tag(MakeValueTypedDataTag(ValueTypeTag::Bool)),
      inner(boolValue) {}

  [[nodiscard]] constexpr inline ValueTypeTag GetTag() const noexcept {
    return static_cast<ValueTypeTag>(tag & VALUE_TYPE_TAG_MASK);
  }

  [[nodiscard]] constexpr inline int64_t GetAsInt() const noexcept {
    assert(GetTag() == ValueTypeTag::Int);
    return inner.intValue;
  }

  [[nodiscard]] constexpr inline double GetAsFloat() const noexcept {
    assert(GetTag() == ValueTypeTag::Float);
    return inner.floatValue;
  }

  [[nodiscard]] constexpr inline char32_t GetAsChar() const noexcept {
    assert(GetTag() == ValueTypeTag::Char);
    return inner.charValue;
  }

  [[nodiscard]] constexpr inline bool GetAsBool() const noexcept {
    assert(GetTag() == ValueTypeTag::Bool);
    return inner.boolValue;
  }

  [[nodiscard]] constexpr inline uint64_t GetRepr() const noexcept {
    return inner.repr;
  }
};

struct alignas(8) WrapperHeader {
  uint32_t refCount;
  uint8_t ownershipInfo;
  uint8_t gcInfo;
  uint8_t dataOffset;
  uint8_t ownershipInfo2;

  WrapperHeader() = delete;
  WrapperHeader(const WrapperHeader&) = delete;
  WrapperHeader(WrapperHeader&&) = delete;

  WrapperHeader& operator=(const WrapperHeader&) = delete;
  WrapperHeader& operator=(WrapperHeader&&) = delete;
};

union Value {
  WidePointer widePointer;
  ValueTypedData valueTypedData;

  constexpr inline explicit Value(WidePointer wide_pointer) noexcept
    : widePointer(wide_pointer) {}

  constexpr inline explicit Value(ValueTypedData value_typed_data) noexcept
    : valueTypedData(value_typed_data) {}
};

extern "C" {

[[noreturn]] void pr47_al31fu_rs_rust_panic();

bool
pr47_al31fu_rs_poll_fut(WidePointer wide_ptr,
                        std::array<Value* __restrict, 8> *ret_values);

} // extern "C"

} // namespace pr47

#endif // PR47_AL31FU_IMPORTS_H
