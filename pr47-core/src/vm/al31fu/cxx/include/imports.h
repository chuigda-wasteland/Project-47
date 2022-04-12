#ifndef PR47_AL31FU_IMPORTS_H
#define PR47_AL31FU_IMPORTS_H

#include <array>
#include <cstdint>

namespace pr47 {
namespace al31fu {

struct WidePointer {
  size_t ptr;
  size_t trivia;

  WidePointer(uintptr_t ptr, uintptr_t trivia) : ptr(ptr), trivia(trivia) {}
};

struct ValueTypedData {
  size_t tag;
  union {
    int64_t int_value;
    double float_value;
    char32_t char_value;
    bool bool_value;
    uint64_t repr;
  } inner;
};

union Value {
  WidePointer wide_ptr;
  ValueTypedData value_typed;
};

extern "C" {

[[noreturn]] void pr47_al31fu_rs_rust_panic();

bool pr47_al31fu_rs_poll_fut(WidePointer wide_ptr, std::array<Value*, 8> *ret_values);

} // extern "C"
} // namespace al31fu
} // namespace pr47

#endif // PR47_AL31FU_IMPORTS_H
