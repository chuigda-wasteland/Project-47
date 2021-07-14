#[cfg(not(debug_assertions))]
use unchecked_unwrap::UncheckedUnwrap;

use crate::data::Value;

#[cfg(debug_assertions)]
#[derive(Copy, Clone)]
pub struct StackSlice(*mut [Option<Value>]);

#[cfg(debug_assertions)]
impl StackSlice {
    pub unsafe fn set_value(&mut self, idx: usize, value: Value) {
        (*self.0)[idx].replace(value);
    }

    pub unsafe fn get_value(&mut self, idx: usize) -> Value {
        (*self.0)[idx].unwrap()
    }
}

#[cfg(not(debug_assertions))]
#[derive(Copy, Clone)]
pub struct StackSlice(*mut [Value]);

#[cfg(not(debug_assertions))]
impl StackSlice {
    pub unsafe fn set_value(&mut self, idx: usize, value: Value) {
        let dest: &mut Value = (*self.0).get_unchecked_mut(idx);
        *dest = value;
    }

    pub unsafe fn get_value(&mut self, idx: usize) -> Value {
        let src: &Value = (*self.0).get_unchecked(idx);
        *src
    }
}

#[derive(Debug)]
pub struct FrameInfo<'a> {
    pub frame_start: usize,
    pub frame_end: usize,
    pub ret_value_locs: &'a [usize],
    pub ret_addr: usize
}

impl<'a> FrameInfo<'a> {
    pub fn new(
        frame_start: usize,
        frame_end: usize,
        ret_value_locs: &'a [usize],
        ret_addr: usize
    ) -> Self {
        Self {
            frame_start,
            frame_end,
            ret_value_locs,
            ret_addr
        }
    }
}

#[cfg(debug_assertions)]
pub struct Stack<'a> {
    pub values: Vec<Option<Value>>,
    pub frames: Vec<FrameInfo<'a>>
}

#[cfg(debug_assertions)]
impl<'a> Stack<'a> {
    pub fn new() -> Self {
        Self {
            values: Vec::with_capacity(64),
            frames: Vec::with_capacity(4)
        }
    }

    pub unsafe fn ext_func_call_grow_stack(
        &mut self,
        frame_size: usize,
        args: &[Value]
    ) -> StackSlice {
        assert_eq!(self.values.len(), 0);
        assert_eq!(self.frames.len(), 0);

        self.values.resize(frame_size, None);
        for (i /*: usize*/, arg /*: &Value*/) in args.iter().enumerate() {
            self.values[i].replace(*arg);
        }
        self.frames.push(FrameInfo::new(0, frame_size, &[], 0));
        StackSlice(&mut self.values[..] as *mut [Option<Value>])
    }

    pub unsafe fn func_call_grow_stack(
        &mut self,
        frame_size: usize,
        arg_locs: &[usize],
        ret_value_locs: &'a [usize],
        ret_addr: usize
    ) -> StackSlice {
        let this_frame: &FrameInfo = self.frames.last().unwrap();
        let (this_frame_start, this_frame_end): (usize, usize)
            = (this_frame.frame_start, this_frame.frame_end);

        assert_eq!(this_frame_end, self.values.len());
        let new_frame_end: usize = this_frame_end + frame_size;
        self.values.resize(new_frame_end, None);
        self.frames.push(FrameInfo::new(this_frame_end, new_frame_end, ret_value_locs, ret_addr));
        let mut old_slice: StackSlice =
            StackSlice(&mut self.values[this_frame_start..this_frame_end] as *mut _);
        let mut new_slice: StackSlice =
            StackSlice(&mut self.values[this_frame_end..new_frame_end] as *mut _);
        for (i /*: usize*/, arg_loc/*: &usize*/) in arg_locs.iter().enumerate() {
            new_slice.set_value(i, old_slice.get_value(*arg_loc));
        }
        new_slice
    }

    pub unsafe fn done_func_call_shrink_stack(
        &mut self,
        ret_values: &[usize]
    ) -> Option<(StackSlice, usize)> {
        let frame_count: usize = self.frames.len();
        if frame_count == 1 {
            return None;
        }

        let this_frame: &FrameInfo = &self.frames[frame_count - 1];
        let prev_frame: &FrameInfo = &self.frames[frame_count - 2];
        assert_eq!(prev_frame.frame_end, this_frame.frame_start);
        let mut this_slice =
            StackSlice(&mut self.values[this_frame.frame_start..this_frame.frame_end] as *mut _);
        let mut prev_slice =
            StackSlice(&mut self.values[prev_frame.frame_start..prev_frame.frame_end] as *mut _);
        assert_eq!(ret_values.len(), this_frame.ret_value_locs.len());
        if ret_values.len() != 0 {
            if ret_values.len() == 1 {
                let from: usize = this_frame.ret_value_locs[0];
                let dest: usize = ret_values[0];
                prev_slice.set_value(dest, this_slice.get_value(from));
            } else {
                for (ret_value /*: &usize*/, ret_value_loc /*: &usize*/) in
                    ret_values.iter().zip(this_frame.ret_value_locs)
                {
                    prev_slice.set_value(*ret_value_loc, this_slice.get_value(*ret_value))
                }
            }
        }
        let ret_addr: usize = this_frame.ret_addr;
        self.values.truncate(prev_frame.frame_end);
        self.frames.pop().unwrap();
        Some((prev_slice, ret_addr))
    }
}

#[cfg(not(debug_assertions))]
pub struct Stack<'a> {
    pub values: Vec<Value>,
    pub frames: Vec<FrameInfo<'a>>
}

#[cfg(not(debug_assertions))]
impl<'a> Stack<'a> {
    pub fn new() -> Self {
        Self {
            values: Vec::with_capacity(64),
            frames: Vec::with_capacity(4)
        }
    }

    pub unsafe fn ext_func_call_grow_stack(
        &mut self,
        frame_size: usize,
        args: &[Value]
    ) -> StackSlice {
        self.values.resize(frame_size, Value::new_null());
        for (i /*: usize*/, arg /*: &Value*/) in args.iter().enumerate() {
            let dest: &mut Value = self.values.get_unchecked_mut(i);
            *dest = *arg;
        }
        self.frames.push(FrameInfo::new(0, frame_size, &[], 0));
        StackSlice(&mut self.values[..] as *mut _)
    }

    pub unsafe fn func_call_grow_stack(
        &mut self,
        frame_size: usize,
        arg_locs: &[usize],
        ret_value_locs: &'a [usize],
        ret_addr: usize
    ) -> StackSlice {
        let this_frame: &FrameInfo = self.frames.last().unchecked_unwrap();
        let (this_frame_start, this_frame_end): (usize, usize)
            = (this_frame.frame_start, this_frame.frame_end);
        let new_frame_end: usize = this_frame_end + frame_size;
        self.values.resize(new_frame_end, Value::new_null());
        self.frames.push(FrameInfo::new(this_frame_end, new_frame_end, ret_value_locs, ret_addr));
        let mut old_slice: StackSlice =
            StackSlice(&mut self.values[this_frame_start..this_frame_end] as *mut _);
        let mut new_slice: StackSlice =
            StackSlice(&mut self.values[this_frame_end..new_frame_end] as *mut _);
        for (i /*: usize*/, arg_loc/*: &usize*/) in arg_locs.iter().enumerate() {
            new_slice.set_value(i, old_slice.get_value(*arg_loc));
        }
        new_slice
    }

    pub unsafe fn done_func_call_shrink_stack(
        &mut self,
        ret_values: &[usize]
    ) -> Option<(StackSlice, usize)> {
        let frame_count = self.frames.len();
        if frame_count == 1 {
            return None;
        }

        let this_frame: &FrameInfo = self.frames.get_unchecked(frame_count - 1);
        let prev_frame: &FrameInfo = self.frames.get_unchecked(frame_count - 2);
        let mut this_slice: StackSlice =
            StackSlice(&mut self.values[this_frame.frame_start..this_frame.frame_end] as *mut _);
        let mut prev_slice: StackSlice =
            StackSlice(&mut self.values[prev_frame.frame_start..prev_frame.frame_end] as *mut _);

        if ret_values.len() != 0 {
            if ret_values.len() == 1 {
                let from: usize = *this_frame.ret_value_locs.get_unchecked(0);
                let dest: usize = *ret_values.get_unchecked(0);
                prev_slice.set_value(dest, this_slice.get_value(from));
            } else {
                for (ret_value /*: &usize*/, ret_value_loc /*: &usize*/) in
                    ret_values.iter().zip(this_frame.ret_value_locs)
                {
                    prev_slice.set_value(*ret_value_loc, this_slice.get_value(*ret_value))
                }
            }
        }
        let ret_addr: usize = this_frame.ret_addr;
        self.values.truncate(prev_frame.frame_end);
        self.frames.pop().unchecked_unwrap();
        Some((prev_slice, ret_addr))
    }
}
