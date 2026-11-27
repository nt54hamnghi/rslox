//! This module provides a managed heap for dynamically allocated
//! runtime data.
//!
//! Strings are handled separately by interning them. The `Str` type
//! is a handle to an interned string and can only be created through
//! one of two allocation functions:
//!
//! - `alloc_str`: A module-level function that does not require a
//!   heap instance, intended for use by the compiler.
//! - `Heap::alloc_str`: An instance method intended for use by the VM.
//!
//! Both functions share the same thread-local string table. Since
//! compilation and execution occur sequentially, the compiler and
//! the VM never access the table concurrently.

use std::cell::RefCell;
use std::collections::HashSet;
use std::thread::LocalKey;

use string::Str;

thread_local! {
    static STRINGS: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
}

type StringTable = LocalKey<RefCell<HashSet<String>>>;

pub fn alloc_str(s: impl Into<String>) -> Str {
    intern_str(&STRINGS, s)
}

fn intern_str(strings: &'static StringTable, s: impl Into<String>) -> Str {
    let s = s.into();
    strings.with_borrow_mut(|table| {
        let stored = match table.get(&s) {
            Some(stored) => stored,
            None => {
                table.insert(s.clone());
                table.get(&s).unwrap()
            }
        };
        Str {
            ptr: stored.as_str(),
        }
    })
}

#[derive(Debug)]
pub struct Heap {
    strings: &'static StringTable,
}

impl Heap {
    pub fn new() -> Self {
        Self { strings: &STRINGS }
    }

    pub fn alloc_str(&mut self, s: impl Into<String>) -> Str {
        intern_str(self.strings, s)
    }
}

pub mod string {
    use std::fmt::Display;

    /// A copyable handle to a string stored in the interned string table.
    ///
    /// An `Str` instance can only be created through the `alloc_str`
    /// functions in the heap module, which intern the given string slice
    /// in the managed heap.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Str {
        pub(super) ptr: *const str,
    }

    impl Str {
        pub fn as_str(&self) -> &str {
            // SAFETY:
            // There's no way to change where the pointer points to, and the
            // byte buffer it's pointing to remains at a stable address.
            //
            // The buffer is owned by a String stored in the interned string
            // table. Since interned strings are never modified or removed,
            // the buffer always contains valid UTF-8. Even if the underlying
            // HashSet resizes, only the String structs are moved, not their
            // byte buffers.
            //
            // The string table is only dropped when the thread terminates.
            unsafe { &*self.ptr }
        }

        pub fn len(&self) -> usize {
            self.as_str().len()
        }
    }

    impl AsRef<str> for Str {
        fn as_ref(&self) -> &str {
            self.as_str()
        }
    }

    impl Display for Str {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            Display::fmt(self.as_str(), f)
        }
    }
}
