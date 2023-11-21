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

    fn print_revision(&self) -> String {
        format!("{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CookingRevisionEntity {
    pub(crate) entity_id: u16,
    pub(crate) property: CookingRevisionEntityProperty,
}

impl CookingRevisionEntity {
    pub(crate) fn new(entity_id: u16, property: CookingRevisionEntityProperty) -> Self {
        Self {
            entity_id,
            property,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum CookingRevisionEntityProperty {
    Add(CookingIngredient),
    Sub(CookingIngredient),
}
