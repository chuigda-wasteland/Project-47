#ifndef PR47_AL31FU_ALIGNED_ALLOC_H
#define PR47_AL31FU_ALIGNED_ALLOC_H

#include <cstddef>
#include <new>

#include "imports.h"

namespace pr47::al31fu {

template <typename T>
inline void* aligned_alloc_panic(size_t count) noexcept {
#ifdef PR47_AL31FU_32BIT
  // TODO: check overflow
#endif

  void *ret = static_cast<void*>(new (std::nothrow) size_t[count]);
  if (!ret) {
    pr47_al31fu_rs_rust_panic();
  }
  return ret;
}

inline void release_aligned_alloc(void *ptr) noexcept {
  delete[] static_cast<size_t*>(ptr);
}

} // namespace pr47::al31fu

#endif // PR47_AL31FU_ALIGNED_ALLOC_H
