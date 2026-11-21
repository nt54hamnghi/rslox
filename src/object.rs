use slotmap::new_key_type;

new_key_type! {
    pub struct ObjectId;
}

#[derive(Debug, Clone)]
pub enum Object {
    String(String),
}

impl From<String> for Object {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}
