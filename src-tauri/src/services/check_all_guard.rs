use std::sync::atomic::{AtomicBool, Ordering};

static CHECK_ALL_RUNNING: AtomicBool = AtomicBool::new(false);

pub struct CheckAllGuard;

impl CheckAllGuard {
    pub fn try_acquire() -> Option<Self> {
        if CHECK_ALL_RUNNING.swap(true, Ordering::AcqRel) {
            None
        } else {
            Some(Self)
        }
    }
}

impl Drop for CheckAllGuard {
    fn drop(&mut self) {
        CHECK_ALL_RUNNING.store(false, Ordering::Release);
    }
}
