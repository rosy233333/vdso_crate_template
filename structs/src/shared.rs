use core::sync::atomic::AtomicUsize;

#[repr(C)]
pub struct VvarData {
    pub example: SharedExample,
}

impl VvarData {
    pub fn new() -> Self {
        VvarData {
            example: SharedExample {
                i: AtomicUsize::new(42),
            },
        }
    }
}

#[repr(C)]
pub struct SharedExample {
    pub i: AtomicUsize,
}
