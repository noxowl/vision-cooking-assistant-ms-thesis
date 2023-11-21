use std::fmt::{self, Debug, Formatter};

pub(crate) mod generic_revision;
pub(crate) mod cooking_revision;

pub(crate) trait Revision: Send {
    fn clone_box(&self) -> Box<dyn Revision>;
    fn as_any(&self) -> &dyn std::any::Any;

    fn print_revision(&self) -> String;
}

impl Clone for Box<dyn Revision> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl Debug for dyn Revision {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.print_revision().fmt(f)
    }
}

impl PartialEq for dyn Revision {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

