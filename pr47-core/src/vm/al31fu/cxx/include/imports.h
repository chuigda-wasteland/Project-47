#ifndef PR47_AL31FU_IMPORTS_H
#define PR47_AL31FU_IMPORTS_H

#include <array>
#include <bit>
#include <cassert>
#include <cstdint>

namespace pr47::al31fu {

struct WidePointer {
  std::size_t ptr;
  std::size_t trivia;

  constexpr inline WidePointer(uintptr_t ptr, uintptr_t trivia) noexcept
    : ptr(ptr), trivia(trivia) {}

  constexpr inline bool operator==(const WidePointer& other) const noexcept {
    return ptr == other.ptr && trivia == other.trivia;
  }
};

constexpr uint8_t TAG_BITS_MASK = 0b00'000'111;
constexpr std::size_t TAG_BITS_MASK_USIZE =
    static_cast<std::size_t>(TAG_BITS_MASK);
constexpr std::size_t PTR_BITS_MASK_USIZE = ~TAG_BITS_MASK_USIZE;
constexpr uint8_t VALUE_TYPE_MASK = 0b00'000'001;
constexpr uint8_t GENERIC_TYPE_MASK = 0b00'000'010;
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

//                                                  G R W M C O
constexpr std::uint8_t OWN_INFO_GLOBASL_MASK = 0b00'1'0'0'0'0'0;
constexpr std::uint8_t OWN_INFO_READ_MASK    = 0b00'0'1'0'0'0'0;
constexpr std::uint8_t OWN_INFO_WRITE_MASK   = 0b00'0'0'1'0'0'0;
constexpr std::uint8_t OWN_INFO_MOVE_MASK    = 0b00'0'0'0'1'0'0;
constexpr std::uint8_t OWN_INFO_COLLECT_MASK = 0b00'0'0'0'0'1'0;
constexpr std::uint8_t OWN_INFO_OWN_MASK     = 0b00'0'0'0'0'0'1;

struct alignas(8) WrapperHeader {
  std::uint32_t refCount;
  std::uint8_t ownershipInfo;
  std::uint8_t gcInfo;
  std::uint8_t dataOffset;
  std::uint8_t ownershipInfo2;

  WrapperHeader() = delete;
  WrapperHeader(const WrapperHeader&) = delete;
  WrapperHeader(WrapperHeader&&) = delete;

  WrapperHeader& operator=(const WrapperHeader&) = delete;
  WrapperHeader& operator=(WrapperHeader&&) = delete;
};

union Value {
  WidePointer widePointer;
  ValueTypedData valueTypedData;
  WrapperHeader *wrapperHeader;

  constexpr inline explicit Value(WidePointer widePointer) noexcept
    : widePointer(widePointer) {}

  constexpr inline explicit Value(ValueTypedData valueTypedData) noexcept
    : valueTypedData(valueTypedData) {}

  constexpr inline explicit Value(int64_t value) noexcept
    : valueTypedData(value) {}

  constexpr inline explicit Value(double value) noexcept
    : valueTypedData(value) {}

  constexpr inline explicit Value(char32_t value) noexcept
    : valueTypedData(value) {}

  constexpr inline explicit Value(bool value) noexcept
    : valueTypedData(value) {}

  constexpr inline static Value CreateNull() noexcept {
    return Value(WidePointer(0, 0));
  }

  constexpr inline bool IsNull() const noexcept {
    return !this->widePointer.ptr;
  }

  constexpr inline bool IsValue() const noexcept {
    return (this->widePointer.ptr & VALUE_TYPE_MASK) != 0;
  }

  constexpr inline bool IsReference() const noexcept {
    return (this->widePointer.ptr & VALUE_TYPE_MASK) == 0;
  }

  constexpr inline bool IsContainer() const noexcept {
    return (this->widePointer.ptr & GENERIC_TYPE_MASK) != 0;
  }

  constexpr inline std::size_t GetUntaggedPtr() const noexcept {
    return (this->widePointer.ptr & PTR_BITS_MASK_USIZE);
  }

  constexpr inline std::uint32_t GetRefCount() const noexcept {
    return std::bit_cast<WrapperHeader const*>(this->GetUntaggedPtr())
        ->refCount;
  }

  constexpr inline std::uint32_t GetRefCountNorm() const noexcept {
    return this->wrapperHeader->refCount;
  }

  constexpr inline void IncrRefCount() noexcept {
    std::bit_cast<WrapperHeader*>(this->GetUntaggedPtr())->refCount += 1;
  }

  constexpr inline void IncrRefCountNorm() noexcept {
    this->wrapperHeader->refCount += 1;
  }

  constexpr inline void DecrRefCount() noexcept {
    std::bit_cast<WrapperHeader*>(this->GetUntaggedPtr())->refCount -= 1;
  }

  constexpr inline void DecrRefCountNorm() noexcept {
    this->wrapperHeader->refCount -= 1;
  }

  constexpr inline std::uint8_t GetOwnershipInfo() const noexcept {
    return std::bit_cast<WrapperHeader const*>(this->GetUntaggedPtr())
        ->ownershipInfo;
  }

  constexpr inline std::uint8_t GetOwnershipInfoNorm() const noexcept {
    return this->wrapperHeader->ownershipInfo;
  }

  constexpr inline void SetOwnershipInfo(std::uint8_t info) noexcept {
    std::bit_cast<WrapperHeader*>(this->GetUntaggedPtr())->ownershipInfo = info;
  }

  constexpr inline void SetOwnershipInfoNorm(std::uint8_t info) noexcept {
    this->wrapperHeader->ownershipInfo = info;
  }

  constexpr inline void BackupOwnershipInfo() noexcept {
    WrapperHeader* header =
        std::bit_cast<WrapperHeader*>(this->GetUntaggedPtr());
    header->ownershipInfo2 = header->ownershipInfo;
  }

  constexpr inline void BackupOwnershipInfoNorm() noexcept {
    this->wrapperHeader->ownershipInfo2 = this->wrapperHeader->ownershipInfo;
  }

  constexpr inline void ResetOwnershipInfo() noexcept {
    WrapperHeader* header =
        std::bit_cast<WrapperHeader*>(this->GetUntaggedPtr());
    header->ownershipInfo = header->ownershipInfo2;
  }

  constexpr inline void ResetOwnershipInfoNorm() noexcept {
    this->wrapperHeader->ownershipInfo = this->wrapperHeader->ownershipInfo2;
  }

  constexpr inline WidePointer GetAsDynBase() const noexcept {
    return WidePointer(this->GetUntaggedPtr(), this->widePointer.trivia);
  }

  constexpr void* GetAsMutPtr() const noexcept {
    std::size_t untaggedPtr = this->GetUntaggedPtr();
    WrapperHeader* header = std::bit_cast<WrapperHeader*>(untaggedPtr);
    std::size_t dataOffset = static_cast<std::size_t>(header->dataOffset);
    if (header->ownershipInfo & OWN_INFO_OWN_MASK) {
      return static_cast<void*>(header + dataOffset);
    } else {
      void **pPtr = std::bit_cast<void**>(header + dataOffset);
      return *pPtr;
    }
  }
};

extern "C" {

[[noreturn]] void pr47_al31fu_rs_rust_panic();

bool
pr47_al31fu_rs_poll_fut(WidePointer wide_ptr,
                        std::array<Value* __restrict, 8> *ret_values);

} // extern "C"

} // namespace pr47::al31fu

#endif // PR47_AL31FU_IMPORTS_H
