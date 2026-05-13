use std::fmt::Debug;
use std::{any::Any, cell::RefCell};

use slotmap::{SlotMap, new_key_type};

new_key_type! { pub(super) struct NodeId; }

#[derive(Debug)]
pub struct Context {
    pub(super) nodes: RefCell<SlotMap<NodeId, Box<dyn Any>>>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            nodes: RefCell::new(SlotMap::with_key()),
        }
    }
}
