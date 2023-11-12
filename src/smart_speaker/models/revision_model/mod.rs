use std::fmt::{self, Debug, Formatter};

pub(crate) mod generic_revision;
pub(crate) mod cooking_revision;

pub(crate) trait Revision: Send {
    fn clone_box(&self) -> Box<dyn Revision>;
    fn as_any(&self) -> &dyn std::any::Any;
}

impl Revision for Box<dyn Revision> {
    fn clone_box(&self) -> Box<dyn Revision> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Debug for dyn Revision {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Revision")
    }
}

impl PartialEq for dyn Revision {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Clone for Box<dyn Revision> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
