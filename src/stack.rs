use std::fmt::Debug;
use std::mem::MaybeUninit;
use std::slice;

// Representation invariants:
// 1. `top <= N`
// 2. Slots in `inner[..top]` contain initialized `T` values.
// 3. Slots in `inner[top..]` are treated as uninitialized and
// must not be read or dropped as `T`.
// 4. Each initialized value is owned by exactly one live stack
// slot. Popping transfers ownership to the caller and removes
// the slot from the initialized prefix.
pub struct Stack<T, const N: usize> {
    inner: [MaybeUninit<T>; N],
    top: usize,
}

impl<T, const N: usize> Stack<T, N> {
    pub const fn new() -> Self {
        Stack {
            inner: [const { MaybeUninit::uninit() }; N],
            top: 0,
        }
    }

    pub fn as_slice(&self) -> &[T] {
        debug_assert!(self.top <= N);
        // SAFETY:
        // `MaybeUninit<T>` has the same size and alignment as `T`, and
        // every slot below `top` contains an initialized `T`.
        //
        // The returned slice borrows `self`, so the memory cannot be
        // mutated/dropped while the slice is alive.
        unsafe { slice::from_raw_parts(self.inner.as_ptr().cast(), self.top) }
    }

    pub fn len(&self) -> usize {
        self.top
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        self.top = 0;
    }

    pub fn push(&mut self, value: T) -> Result<(), StackOverflow> {
        self.inner
            .get_mut(self.top)
            .ok_or(StackOverflow(N))?
            .write(value);

        self.top += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        self.top -= 1;

        // SAFETY:
        // By the type invariant, after `top` is decremented, the slot at
        // `top` is considered uninitialized.
        //
        // `assume_init_read` returns a bitwise copy whose ownership is
        // transferred to the caller. A subsequent push will overwrite the
        // slot at `top` with `MaybeUninit::write`, which doesn't drop the
        // existing `T`, so there won't be a double drop.
        let value = unsafe { self.inner[self.top].assume_init_read() };
        Some(value)
    }

    /// Returns a reference to the value at `distance` from the top of the stack.
    ///
    /// A distance of `0` returns the top value, `1` returns the value directly
    /// below it, and so on. Returns `None` if `distance` is outside the stack's
    /// initialized values.
    pub fn peek(&self, distance: usize) -> Option<&T> {
        if distance >= self.len() {
            return None;
        }

        // SAFETY:
        // Since `distance < top`, `top - 1 - distance` indexes a slot in
        // `inner[..top]`, which is initialized by the type invariant.
        let value = unsafe { self.inner[self.top - 1 - distance].assume_init_ref() };
        Some(value)
    }
}

impl<T: Debug, const N: usize> Debug for Stack<T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.as_slice())
    }
}

impl<T, const N: usize> Drop for Stack<T, N> {
    fn drop(&mut self) {
        // SAFETY: Visit only the initialized values below `top`
        // so that moved/popped values are not dropped twice.
        for elem in &mut self.inner[0..self.top] {
            unsafe {
                elem.assume_init_drop();
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("stack overflow: exceeded capacity of {0} values")]
pub struct StackOverflow(usize);

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use rstest::rstest;

    use super::{Stack, StackOverflow};

    #[test]
    fn new_stack_is_empty() {
        let stack = Stack::<i32, 3>::new();

        assert_eq!(stack.len(), 0);
        assert!(stack.is_empty());
    }

    #[rstest]
    #[case(&[10])]
    #[case(&[10, 20])]
    #[case(&[10, 20, 30])]
    fn push_updates_state_and_contents(#[case] values: &[i32]) {
        let mut stack = Stack::<i32, 3>::new();

        for &value in values {
            stack.push(value).unwrap();
        }

        assert!(!stack.is_empty());
        assert_eq!(stack.len(), values.len());
        assert_eq!(stack.as_slice(), values);
    }

    #[test]
    fn push_returns_error_when_full() {
        let mut stack = Stack::<i32, 3>::new();
        for value in [10, 20, 30] {
            stack.push(value).unwrap();
        }
        let result = stack.push(40);

        assert!(result.is_err());
        assert!(!stack.is_empty());
        assert_eq!(stack.len(), 3);
        assert_eq!(stack.as_slice(), &[10, 20, 30]);
    }

    #[test]
    fn push_returns_error_with_zero_capacity() {
        let mut stack = Stack::<i32, 0>::new();
        let result = stack.push(10);

        assert!(result.is_err());
        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
        assert_eq!(stack.pop(), None);
        assert_eq!(stack.as_slice(), &[]);
    }

    #[rstest]
    #[case(&[], &[])]
    #[case(&[10], &[10])]
    #[case(&[10, 20], &[20, 10])]
    #[case(&[10, 20, 30], &[30, 20, 10])]
    fn pop_returns_values_in_lifo_order(#[case] values: &[i32], #[case] expected_order: &[i32]) {
        let mut stack = Stack::<i32, 3>::new();
        for &value in values {
            stack.push(value).unwrap();
        }

        let mut out = Vec::with_capacity(expected_order.len());
        for _ in 0..expected_order.len() {
            let value = stack.pop();
            assert!(value.is_some());
            out.push(value.unwrap());
        }

        assert_eq!(stack.pop(), None);
        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
        assert_eq!(stack.as_slice(), &[]);
        assert_eq!(out, expected_order);
    }

    #[rstest]
    #[case(&[])]
    #[case(&[10])]
    #[case(&[10, 20, 30])]
    fn clear_empties_stack_and_allows_reuse(#[case] values: &[i32]) {
        let mut stack = Stack::<i32, 3>::new();
        for &value in values {
            stack.push(value).unwrap();
        }

        stack.clear();

        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
        assert_eq!(stack.as_slice(), &[]);
        assert_eq!(stack.pop(), None);
        assert_eq!(stack.peek(0), None);

        for value in [40, 50, 60] {
            stack.push(value).unwrap();
        }
        assert_eq!(stack.as_slice(), &[40, 50, 60]);
        assert_eq!(stack.pop(), Some(60));
    }

    #[test]
    fn clear_with_zero_capacity() {
        let mut stack = Stack::<i32, 0>::new();
        stack.clear();

        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
        assert_eq!(stack.as_slice(), &[]);
    }

    #[test]
    fn peek_with_zero_capacity() {
        let stack = Stack::<i32, 0>::new();

        assert_eq!(stack.peek(0), None);
        assert_eq!(stack.peek(1), None);
    }

    #[rstest]
    #[case(0, 30)]
    #[case(1, 20)]
    #[case(2, 10)]
    fn peek_borrows_from_top_without_consuming(#[case] distance: usize, #[case] expected: i32) {
        let mut stack = Stack::<i32, 3>::new();
        for value in [10, 20, 30] {
            stack.push(value).unwrap();
        }

        assert_eq!(stack.peek(distance), Some(&expected));
        assert_eq!(stack.len(), 3);
        assert_eq!(stack.as_slice(), &[10, 20, 30]);
    }

    #[test]
    fn peek_borrows_new_top_after_pop() {
        let mut stack = Stack::<i32, 3>::new();
        for value in [10, 20, 30] {
            stack.push(value).unwrap();
        }
        stack.pop().unwrap();

        assert_eq!(stack.peek(0), Some(&20));
    }

    #[test]
    fn peek_borrows_replacement_after_push_reuses_popped_slot() {
        let mut stack = Stack::<i32, 3>::new();
        for value in [10, 20, 30] {
            stack.push(value).unwrap();
        }
        stack.pop().unwrap();
        stack.push(40).unwrap();

        assert_eq!(stack.peek(0), Some(&40));
    }

    #[rstest]
    #[case(&[], 0)]
    #[case(&[], 1)]
    #[case(&[10], 1)]
    #[case(&[10], 2)]
    #[case(&[10, 20, 30], 3)]
    fn peek_returns_none_outside_live_values(#[case] values: &[i32], #[case] distance: usize) {
        let mut stack = Stack::<i32, 3>::new();
        for &value in values {
            stack.push(value).unwrap();
        }

        assert_eq!(stack.peek(distance), None);
        assert_eq!(stack.as_slice(), values);
    }

    #[test]
    fn push_reuses_popped_slot() {
        let mut stack = Stack::<i32, 3>::new();
        stack.push(10).unwrap();
        stack.push(20).unwrap();
        assert_eq!(stack.pop(), Some(20));
        stack.push(30).unwrap();

        assert_eq!(stack.len(), 2);
        assert!(!stack.is_empty());
        assert_eq!(stack.as_slice(), &[10, 30]);
    }

    #[rstest]
    #[case(&[], 0, "[]")]
    #[case(&[10], 0, "[10]")]
    #[case(&[10, 20, 30], 0, "[10, 20, 30]")]
    #[case(&[10, 20, 30], 1, "[10, 20]")]
    #[case(&[10, 20, 30], 3, "[]")]
    fn debug_shows_live_values(
        #[case] values: &[i32],
        #[case] pop_count: usize,
        #[case] expected: &str,
    ) {
        let mut stack = Stack::<i32, 3>::new();
        for &value in values {
            stack.push(value).unwrap();
        }
        for _ in 0..pop_count {
            let _ = stack.pop();
        }

        assert_eq!(format!("{stack:?}"), expected);
    }

    #[rstest]
    #[case(0, "stack overflow: exceeded capacity of 0 values")]
    #[case(3, "stack overflow: exceeded capacity of 3 values")]
    fn overflow_message_includes_capacity(#[case] capacity: usize, #[case] expected: &str) {
        assert_eq!(StackOverflow(capacity).to_string(), expected);
    }

    struct DropCounter<'a>(&'a Cell<usize>);

    impl Drop for DropCounter<'_> {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }

    #[test]
    fn drop_drops_each_value_once() {
        const COUNT: usize = 3;
        let mut stack = Stack::<_, COUNT>::new();
        let drops = [const { Cell::new(0) }; COUNT];
        for d in &drops {
            stack.push(DropCounter(d)).unwrap();
        }
        drop(stack);

        for c in &drops {
            assert_eq!(c.get(), 1);
        }
    }

    #[test]
    fn popped_value_is_not_dropped_twice() {
        let drops = Cell::new(0);
        let mut stack = Stack::<DropCounter<'_>, 1>::new();
        stack.push(DropCounter(&drops)).unwrap();
        drop(stack.pop());
        drop(stack);

        assert_eq!(drops.get(), 1);
    }
}
