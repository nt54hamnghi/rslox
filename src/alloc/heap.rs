use slotmap::SlotMap;

use crate::object::{Object, ObjectId};

const INIT_HEAP_CAPACITY: usize = 256;

pub struct Heap {
    objects: SlotMap<ObjectId, Object>,
}

impl Heap {
    pub fn new() -> Self {
        Self {
            objects: SlotMap::with_capacity_and_key(INIT_HEAP_CAPACITY),
        }
    }

    pub fn alloc(&mut self, obj: impl Into<Object>) -> ObjectId {
        self.objects.insert(obj.into())
    }

    #[allow(unused)]
    pub fn free(&mut self, id: ObjectId) -> Option<Object> {
        self.objects.remove(id)
    }

    pub fn get(&self, id: ObjectId) -> Option<&Object> {
        self.objects.get(id)
    }
}
