//! The one message type components share.
//!
//! A component's popup talks to its own code with its own message enum; the
//! app only needs a single `Message::SipilArch` that carries any of them. The
//! payload is type-erased so adding a component never touches the app's
//! `Message` enum.

use std::any::Any;
use std::fmt;
use std::sync::Arc;

/// A message from a component's popup, addressed by the component's id.
#[derive(Clone)]
pub struct ArchMsg {
    pub id: &'static str,
    payload: Arc<dyn Any + Send + Sync>,
}

impl ArchMsg {
    pub fn new<T: Any + Send + Sync>(id: &'static str, payload: T) -> Self {
        Self { id, payload: Arc::new(payload) }
    }

    /// The payload as `T`, if that is what it is.
    pub fn payload<T: Any>(&self) -> Option<&T> {
        self.payload.downcast_ref::<T>()
    }
}

impl fmt::Debug for ArchMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ArchMsg({})", self.id)
    }
}
