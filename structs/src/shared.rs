#[repr(C)]
pub struct VvarData {
    pub example: SharedExample,
}

impl VvarData {
    pub fn new() -> Self {
        VvarData {
            example: SharedExample { i: 42 },
        }
    }
}

#[repr(C)]
pub struct SharedExample {
    pub i: usize,
}
