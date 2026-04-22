use serde_json::Value;

pub struct SchemaHardener;

impl SchemaHardener {
    pub fn new() -> Self {
        Self
    }

    pub fn harden(&self, _schema: &mut Value) {
        todo!("W-6 implementation")
    }

    pub fn harden_all(&self, schemas: &mut [Value]) {
        for s in schemas.iter_mut() {
            self.harden(s);
        }
    }
}

impl Default for SchemaHardener {
    fn default() -> Self {
        Self::new()
    }
}
