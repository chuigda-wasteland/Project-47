#ifndef PR47_AL31FU_STACK_H
#define PR47_AL31FU_STACK_H

#include "imports.h"

namespace pr47 {
namespace al31fu {

class Stack {
public:
  Stack(size_t initSize) {
    // @todo
  }

  Stack(const Stack&) = delete;
  Stack& operator=(const Stack&) = delete;
  Stack(Stack&&) = delete;
  Stack& operator=(Stack&&) = delete;

private:
  Value *m_Values;
  Value *m_FrameStart;
  Value *m_FrameEnd;
};

} // namespace al31fu
} // namespace pr47

#endif // PR47_AL31FU_STACK_H
