use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Debug;
use std::hash::Hash;

trait NextInt {
    fn next_int(&mut self) -> Self;
}

impl NextInt for isize {
    fn next_int(&mut self) -> Self {
        let ret: isize = *self;
        *self += 1;
        ret
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum BorrowState {
    Own,
    Move,
    AlreadyMoved,
    Borrow,
    BorrowAgain,
    BorrowMut,
    UnBorrow,
    UnBorrowMut
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum ObjectOriginate {
    OwnedByVM,
    SharedFromRust,
    MutSharedFromRust
}

pub const READ: usize = 0;
pub const WRITE: usize = 1;
pub const MOVABLE: usize = 2;
pub const COLLECT: usize = 3;
pub const OWNED: usize = 5;

type StateFlags = [bool; 5];

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum BorrowMark {
    ReadBorrow(Option<StateFlags>),
    WriteBorrow(StateFlags)
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct ObjectState {
    originate: ObjectOriginate,
    borrow_stack: Vec<BorrowState>,

    state_flags: StateFlags,
    borrow_marks: Vec<BorrowMark>
}

impl ObjectState {
    fn owned_by_vm() -> Self {
        Self {
            originate: ObjectOriginate::OwnedByVM,
            borrow_stack: vec![BorrowState::Own],

            state_flags: [true, true, true, true, true],
            borrow_marks: vec![]
        }
    }

    fn shared_from_rust() -> Self {
        Self {
            originate: ObjectOriginate::SharedFromRust,
            borrow_stack: vec![BorrowState::Borrow],

            state_flags: [true, false, false, true, false],
            borrow_marks: vec![]
        }
    }

    fn mut_shared_from_rust() -> Self {
        Self {
            originate: ObjectOriginate::MutSharedFromRust,
            borrow_stack: vec![],

            state_flags: [true, true, false, true, false],
            borrow_marks: vec![]
        }
    }

    fn reduce_borrow_stack(&mut self) {
        use self::BorrowState::*;

        if self.borrow_stack.len() >= 2 {
            let len: usize = self.borrow_stack.len();
            if self.borrow_stack[len - 1] == UnBorrow && self.borrow_stack[len - 2] == Borrow {
                self.borrow_stack.pop();
                self.borrow_stack.pop();
            } else if self.borrow_stack[len - 1] == UnBorrowMut &&
                self.borrow_stack[len - 2] == BorrowMut {
                self.borrow_stack.pop();
                self.borrow_stack.pop();
            } else if self.borrow_stack[len - 1] == Borrow &&
                [Borrow, BorrowAgain].contains(&self.borrow_stack[len - 2])
            {
                self.borrow_stack.pop();
                self.borrow_stack.pop();
                self.borrow_stack.push(BorrowAgain);
            } else if self.borrow_stack[len - 1] == UnBorrow &&
                self.borrow_stack[len - 2] == BorrowAgain
            {
                self.borrow_stack.pop();
                self.borrow_stack.pop();
                self.borrow_stack.push(Borrow);
            } else if self.borrow_stack[len - 1] == Move &&
                self.borrow_stack[len - 2] == Own {
                self.borrow_stack.pop();
                self.borrow_stack.pop();
                self.borrow_stack.push(AlreadyMoved);
            }
        }
    }
}

impl ObjectState {
    fn try_borrow(&self) -> Option<ObjectState> {
        if !self.state_flags[READ] {
            return None;
        }

        let mut self_clone: ObjectState = self.clone();

        let new_borrow_mark: BorrowMark = BorrowMark::ReadBorrow(if self.state_flags[WRITE] {
            self_clone.state_flags[WRITE] = false;
            Some(self.state_flags)
        } else {
            None
        });
        if let Some(borrow_mark) = self_clone.borrow_marks.last() {
            if *borrow_mark != new_borrow_mark {
                self_clone.borrow_marks.push(new_borrow_mark);
            }
        } else {
            self_clone.borrow_marks.push(new_borrow_mark);
        }

        self_clone.borrow_stack.push(BorrowState::Borrow);
        self_clone.reduce_borrow_stack();
        Some(self_clone)
    }

    fn try_borrow_mut(&self) -> Option<ObjectState> {
        if !self.state_flags[WRITE] {
            return None;
        }

        let mut self_clone: ObjectState = self.clone();
        self_clone.state_flags[READ] = false;
        self_clone.state_flags[WRITE] = false;
        self_clone.state_flags[MOVABLE] = false;
        self_clone.borrow_marks.push(BorrowMark::WriteBorrow(self.state_flags));
        self_clone.borrow_stack.push(BorrowState::BorrowMut);
        self_clone.reduce_borrow_stack();
        Some(self_clone)
    }

    fn try_move(&self) -> Option<ObjectState> {
        if !self.state_flags[MOVABLE] || !self.state_flags[READ] || !self.state_flags[WRITE] {
            return None;
        }

        let mut self_clone: ObjectState = self.clone();
        self_clone.state_flags = [false, false, false, true, false];
        self_clone.borrow_stack.push(BorrowState::Move);
        self_clone.reduce_borrow_stack();
        Some(self_clone)
    }

    fn try_unborrow(&self) -> Option<ObjectState> {
        if self.borrow_marks.len() == 0 {
            return None;
        }

        let mut self_clone: ObjectState = self.clone();
        if let BorrowMark::ReadBorrow(maybe_state_flags) = self_clone.borrow_marks.pop().unwrap() {
            if let Some(state_flags) = maybe_state_flags {
                self_clone.state_flags = state_flags;
            }
            self_clone.borrow_stack.push(BorrowState::UnBorrow);
            self_clone.reduce_borrow_stack();
            Some(self_clone)
        } else {
            None
        }
    }

    fn try_unborrow_mut(&self) -> Option<ObjectState> {
        if self.borrow_marks.len() == 0 {
            return None;
        }

        let mut self_clone: ObjectState = self.clone();
        if let BorrowMark::WriteBorrow(state_flags) = self_clone.borrow_marks.pop().unwrap() {
            self_clone.state_flags = state_flags;
            self_clone.borrow_stack.push(BorrowState::UnBorrowMut);
            self_clone.reduce_borrow_stack();
            Some(self_clone)
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct BFSResult<'a, S: Clone + Debug + Eq + Hash> {
    searched_states: HashMap<S, isize>,
    transformations: HashSet<(isize, isize, &'a str)>
}

fn breath_first_search<'a, I, S, F>(
    initial_states: I,
    transformers: &[(F, &'a str)]
) -> BFSResult<'a, S>
    where I: IntoIterator<Item = S>,
          S: Clone + Debug + Eq + Hash,
          F: Fn(&S) -> Option<S>
{
    let mut searched_states: HashMap<S, isize> = HashMap::new();
    let mut transformations: HashSet<(isize, isize, &'a str)> = HashSet::new();
    let mut search_queue: VecDeque<(isize, S)> = initial_states
        .into_iter()
        .enumerate()
        .map(|(idx, state)| (idx as isize, state))
        .collect::<VecDeque<_>>();
    let mut state_idx: isize = search_queue.len() as isize;

    for (idx, state) in search_queue.iter() {
        searched_states.insert(state.clone(), *idx);
    }

    while let Some((idx, state)) = search_queue.pop_front() {
        for (transformer, name) in transformers.iter() {
            if let Some(new_state) = transformer(&state) {
                if !searched_states.contains_key(&new_state) {
                    let new_state_idx: isize = state_idx.next_int();
                    searched_states.insert(new_state.clone(), new_state_idx);
                    search_queue.push_back((new_state_idx, new_state));
                    transformations.insert((idx, new_state_idx, name));
                }
            } else {
                transformations.insert((idx, -1, name));
            }
        }
    }

    BFSResult {
        searched_states,
        transformations
    }
}

type SuperFn = for<'r> fn(&'r ObjectState) -> Option<ObjectState>;

fn main() {
    let result = breath_first_search(
        [
            ObjectState::owned_by_vm(),
            ObjectState::shared_from_rust(),
            ObjectState::mut_shared_from_rust()
        ],
        &[
            (ObjectState::try_borrow as SuperFn, "borrow"),
            (ObjectState::try_borrow_mut as SuperFn, "borrow_mut"),
            (ObjectState::try_move as SuperFn, "move"),
            (ObjectState::try_unborrow as SuperFn, "unborrow"),
            (ObjectState::try_unborrow_mut as SuperFn, "unborrow_mut")
        ]
    );

    println!("searched states:");
    for (k, v) in result.searched_states {
        println!("  state[{}] = {:?}", v, k)
    }

    println!("\ntransformations:");
    for transformation in result.transformations {
        println!("  {} |-> {} via {}", transformation.0, transformation.1, transformation.2)
    }
}
