use crate::smart_speaker::models::revision_model::Revision;
use crate::smart_speaker::models::task_model::cooking_task::CookingIngredient;

#[derive(Debug, Clone)]
pub(crate) struct CookingRevision {
    pub(crate) entities: Vec<CookingRevisionEntity>
}

impl CookingRevision {
    pub(crate) fn new(entities: Vec<CookingRevisionEntity>) -> Self {
        CookingRevision {
            entities
        }
    }
}

impl Revision for CookingRevision {
    fn clone_box(&self) -> Box<dyn Revision> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone)]
pub(crate) enum CookingRevisionEntity {
    Add(CookingIngredient),
    Remove(CookingIngredient),
    Replace(CookingIngredient),
}
