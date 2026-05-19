use std::any::Any;
use std::cell::RefCell;
use std::fmt::Debug;

use slotmap::{SlotMap, new_key_type};

use crate::parser::stmt::StmtNode;

#[derive(Debug)]
pub struct Ast {
    pub ctx: &'static Context,
    pub stmts: Vec<StmtNode>,
}

new_key_type! { pub struct NodeId; }

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
