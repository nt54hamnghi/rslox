use std::fmt::Debug;
use std::mem::MaybeUninit;
use std::slice;

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
        debug_assert!(self.len() <= N);
        // SAFETY: MaybeUninit<T> has the same size and alignment as T, and
        // self.inner is non-null and valid for reads for self.len() elements.
        // Every T in the slice is initialized because self.len() increases only
        // when push writes an element and decreases when pop reads one out, so
        // the first self.len() slots are always initialized. The returned slice
        // borrows self, so the memory can't be mutated while the slice is alive.
        unsafe { slice::from_raw_parts(self.inner.as_ptr().cast(), self.len()) }
    }

    pub fn len(&self) -> usize {
        self.top
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
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

        // SAFETY: every slot below self.top is initialized by a prior push, and
        // we have already checked that the stack is non-empty before decrementing
        // self.top, so indexing with it returns an initialized item. Returning a
        // bitwise copy with assume_init_read is safe because the old data will be
        // overwritten by push (via MaybeUninit::write), which doesn't drop the
        // existing T, so we won't have a double drop.
        let value = unsafe { self.inner[self.top].assume_init_read() };
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
        // SAFETY: The first `top` slots contain initialized values.
        // The remaining slots must not be dropped as T
        let len = self.len();
        for elem in &mut self.inner[0..len] {
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
