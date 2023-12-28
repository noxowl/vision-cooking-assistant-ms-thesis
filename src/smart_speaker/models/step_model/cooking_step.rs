use std::fmt::{Debug};
use anyhow::{anyhow, Result};
use handlebars::Handlebars;
use serde_json::json;
use crate::smart_speaker::models::intent_model::IntentCookingMenu;
use crate::smart_speaker::models::step_model::generic_step::{ActionExecutable, ActionTriggerType};
use crate::smart_speaker::models::task_model::{SmartSpeakerTaskResult, SmartSpeakerTaskResultCode};
use crate::smart_speaker::models::vision_model::{DetectableObject, DetectionDetail, DetectionMode, VisionAction, VisionObject};
use crate::smart_speaker::models::message_model::*;
use crate::smart_speaker::models::revision_model::cooking_revision::{CookingRevision, CookingRevisionEntity, CookingRevisionEntityProperty};
use crate::smart_speaker::models::revision_model::Revision;
use crate::smart_speaker::models::task_model::cooking_task::{CookingIngredient, CookingIngredientAmount, CookingIngredientLinkComponent, CookingIngredientName, CookingIngredientTime, SmartSpeakerMaterialProperty};

#[derive(Debug, Clone)]
pub(crate) enum CookingActionDetail {
    None,
    ExplainNonMutableIngredient,
    ExplainMutableIngredient(CookingIngredientLinkComponent),
    ExplainMutableTime(CookingIngredientTime),
    MeasureIngredientSize,
    MeasureCutIngredient,
}


#[derive(Debug, Clone)]
pub(crate) struct ExplainRecipeAction {
    pub(crate) ingredients: Vec<CookingIngredient>,
    pub(crate) detail: CookingActionDetail,
    pub(crate) tts_script: SmartSpeakerI18nText,
    pub(crate) current_content: Option<IntentContent>,
    pub(crate) current_revision: Option<CookingRevision>,
    cancelled: bool,
    repeat_requested: bool,
}

impl ExplainRecipeAction {
    pub(crate) fn new(ingredients: Vec<CookingIngredient>, detail: CookingActionDetail, text: SmartSpeakerI18nText) -> Self {
        ExplainRecipeAction {
            ingredients,
            detail,
            tts_script: text,
            current_content: None,
            current_revision: None,
            cancelled: false,
            repeat_requested: false,
        }
    }
}

impl ActionExecutable for ExplainRecipeAction {
    fn execute(&self) -> Result<SmartSpeakerTaskResult> {
        if self.has_cancelled() {
            return Ok(SmartSpeakerTaskResult::new(
                self.get_action_trigger_type().to_task_type(),
                SmartSpeakerTaskResultCode::Cancelled));
        }
        if self.has_request_repeat() {
            return Ok(SmartSpeakerTaskResult::new(
                self.get_action_trigger_type().to_task_type(),
                SmartSpeakerTaskResultCode::RepeatPrevious));
        }
        let mut reg = Handlebars::new();
        let mut tts_script = self.tts_script.clone();
        match &self.current_content {
            Some(intent) => {
                match &self.detail {
                    CookingActionDetail::ExplainNonMutableIngredient => {
                        tts_script.en = reg.render_template(&self.tts_script.en, &json!({
                            "additional_explain": self.ingredients.iter()
                            .map(|i| {
                                if i.name.to_material_property() == SmartSpeakerMaterialProperty::Solid {
                                    format!("{} {}", i.to_approx_unit_i18n().en, i.name.to_i18n().en)
                                } else {
                                    format!("{}", i.name.to_i18n().en)
                                }
                            }).collect::<Vec<String>>().join(".   .")
                        })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                        tts_script.ja = reg.render_template(&self.tts_script.ja, &json!({
                            "additional_explain": self.ingredients.iter()
                            .map(|i| {
                                if i.name.to_material_property() == SmartSpeakerMaterialProperty::Solid {
                                    format!("{} {}", i.name.to_i18n().ja, i.to_approx_unit_i18n().ja)
                                } else {
                                    format!("{}", i.name.to_i18n().ja)
                                }
                            }).collect::<Vec<String>>().join("、　　　、")
                        })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                        tts_script.zh = reg.render_template(&self.tts_script.zh, &json!({
                            "additional_explain": self.ingredients.iter()
                            .map(|i| {
                                if i.name.to_material_property() == SmartSpeakerMaterialProperty::Solid {
                                    format!("{} {}", i.name.to_i18n().zh, i.to_approx_unit_i18n().zh)
                                } else {
                                    format!("{}", i.name.to_i18n().zh)
                                }
                            }).collect::<Vec<String>>().join("、　　　、")
                        })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                        tts_script.ko = reg.render_template(&self.tts_script.ko, &json!({
                            "additional_explain": self.ingredients.iter()
                            .map(|i| {
                                if i.name.to_material_property() == SmartSpeakerMaterialProperty::Solid {
                                    format!("{} {}", i.name.to_i18n().ko, i.to_approx_unit_i18n().ko)
                                } else {
                                    format!("{}", i.name.to_i18n().ko)
                                }
                            }).collect::<Vec<String>>().join(".   .")
                        })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                    }
                    CookingActionDetail::ExplainMutableIngredient(link) => {
                        match &self.current_revision {
                            None => {
                                tts_script.en = reg.render_template(&self.tts_script.en, &json!({
                                    "salt": self.ingredients.iter().find(|i| i.name == CookingIngredientName::Salt).and_then(|object| Some(object.to_approx_unit_i18n().en)),
                                    "pepper": self.ingredients.iter().find(|i| i.name == CookingIngredientName::Pepper).and_then(|object| Some(object.to_approx_unit_i18n().en)),
                                    "sugar": self.ingredients.iter().find(|i| i.name == CookingIngredientName::Sugar).and_then(|object| Some(object.to_approx_unit_i18n().en)),
                                    "soy_sauce": self.ingredients.iter().find(|i| i.name == CookingIngredientName::SoySauce).and_then(|object| Some(object.to_approx_unit_i18n().en)),
                                    "sesame": self.ingredients.iter().find(|i| i.name == CookingIngredientName::Sesame).and_then(|object| Some(object.to_approx_unit_i18n().en)),
                                    "sesame_oil": self.ingredients.iter().find(|i| i.name == CookingIngredientName::SesameOil).and_then(|object| Some(object.to_approx_unit_i18n().en)),
                                    "carrot": self.ingredients.iter().find(|i| i.name == CookingIngredientName::Carrot).and_then(|object| Some(object.to_approx_unit_i18n().en)),
                                    "mayonnaise": self.ingredients.iter().find(|i| i.name == CookingIngredientName::Mayonnaise).and_then(|object| Some(object.to_approx_unit_i18n().en)),
                                })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                                tts_script.ja = reg.render_template(&self.tts_script.ja, &json!({
                                    "salt": self.ingredients.iter().find(|i| i.name == CookingIngredientName::Salt).and_then(|object| Some(object.to_approx_unit_i18n().ja)),
                                    "pepper": self.ingredients.iter().find(|i| i.name == CookingIngredientName::Pepper).and_then(|object| Some(object.to_approx_unit_i18n().ja)),
                                    "sugar": self.ingredients.iter().find(|i| i.name == CookingIngredientName::Sugar).and_then(|object| Some(object.to_approx_unit_i18n().ja)),
                                    "soy_sauce": self.ingredients.iter().find(|i| i.name == CookingIngredientName::SoySauce).and_then(|object| Some(object.to_approx_unit_i18n().ja)),
                                    "sesame": self.ingredients.iter().find(|i| i.name == CookingIngredientName::Sesame).and_then(|object| Some(object.to_approx_unit_i18n().ja)),
                                    "sesame_oil": self.ingredients.iter().find(|i| i.name == CookingIngredientName::SesameOil).and_then(|object| Some(object.to_approx_unit_i18n().ja)),
                                    "carrot": self.ingredients.iter().find(|i| i.name == CookingIngredientName::Carrot).and_then(|object| Some(object.to_approx_unit_i18n().ja)),
                                    "mayonnaise": self.ingredients.iter().find(|i| i.name == CookingIngredientName::Mayonnaise).and_then(|object| Some(object.to_approx_unit_i18n().ja)),
                                })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                                tts_script.zh = reg.render_template(&self.tts_script.zh, &json!({
                                    "salt": self.ingredients.iter().find(|i| i.name == CookingIngredientName::Salt).and_then(|object| Some(object.to_approx_unit_i18n().zh)),
                                    "pepper": self.ingredients.iter().find(|i| i.name == CookingIngredientName::Pepper).and_then(|object| Some(object.to_approx_unit_i18n().zh)),
                                    "sugar": self.ingredients.iter().find(|i| i.name == CookingIngredientName::Sugar).and_then(|object| Some(object.to_approx_unit_i18n().zh)),
                                    "soy_sauce": self.ingredients.iter().find(|i| i.name == CookingIngredientName::SoySauce).and_then(|object| Some(object.to_approx_unit_i18n().zh)),
                                    "sesame": self.ingredients.iter().find(|i| i.name == CookingIngredientName::Sesame).and_then(|object| Some(object.to_approx_unit_i18n().zh)),
                                    "sesame_oil": self.ingredients.iter().find(|i| i.name == CookingIngredientName::SesameOil).and_then(|object| Some(object.to_approx_unit_i18n().zh)),
                                    "carrot": self.ingredients.iter().find(|i| i.name == CookingIngredientName::Carrot).and_then(|object| Some(object.to_approx_unit_i18n().zh)),
                                    "mayonnaise": self.ingredients.iter().find(|i| i.name == CookingIngredientName::Mayonnaise).and_then(|object| Some(object.to_approx_unit_i18n().zh)),
                                })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                                tts_script.ko = reg.render_template(&self.tts_script.ko, &json!({
                                    "salt": self.ingredients.iter().find(|i| i.name == CookingIngredientName::Salt).and_then(|object| Some(object.to_approx_unit_i18n().ko)),
                                    "pepper": self.ingredients.iter().find(|i| i.name == CookingIngredientName::Pepper).and_then(|object| Some(object.to_approx_unit_i18n().ko)),
                                    "sugar": self.ingredients.iter().find(|i| i.name == CookingIngredientName::Sugar).and_then(|object| Some(object.to_approx_unit_i18n().ko)),
                                    "soy_sauce": self.ingredients.iter().find(|i| i.name == CookingIngredientName::SoySauce).and_then(|object| Some(object.to_approx_unit_i18n().ko)),
                                    "sesame": self.ingredients.iter().find(|i| i.name == CookingIngredientName::Sesame).and_then(|object| Some(object.to_approx_unit_i18n().ko)),
                                    "sesame_oil": self.ingredients.iter().find(|i| i.name == CookingIngredientName::SesameOil).and_then(|object| Some(object.to_approx_unit_i18n().ko)),
                                    "carrot": self.ingredients.iter().find(|i| i.name == CookingIngredientName::Carrot).and_then(|object| Some(object.to_approx_unit_i18n().ko)),
                                    "mayonnaise": self.ingredients.iter().find(|i| i.name == CookingIngredientName::Mayonnaise).and_then(|object| Some(object.to_approx_unit_i18n().ko)),
                                })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                            }
                            Some(rev) => {
                                let mut ingredients_updated = vec![];
                                let mut revision_entities_updated = vec![];
                                self.current_revision.clone().and_then(|mut r| {
                                    r.entities.pop().and_then(
                                        |entity| {
                                            ingredients_updated = link.calc_components_amount_by_main_revision(&entity);
                                            None::<CookingRevision>
                                        });
                                    revision_entities_updated = r.entities;
                                    None::<CookingRevision>
                                });
                                if ingredients_updated.len() == 0 {
                                    return Ok(SmartSpeakerTaskResult::with_tts_and_revision(
                                        self.get_action_trigger_type().to_task_type(),
                                        SmartSpeakerTaskResultCode::StepFailed,
                                        tts_script,
                                        Box::new(rev.clone()),
                                    ));
                                } else {
                                    tts_script.en = reg.render_template(&self.tts_script.en, &json!({
                                        "salt": ingredients_updated.iter().find(|i| i.name == CookingIngredientName::Salt).and_then(|object| Some(object.to_approx_unit_i18n().en)),
                                        "pepper": ingredients_updated.iter().find(|i| i.name == CookingIngredientName::Pepper).and_then(|object| Some(object.to_approx_unit_i18n().en)),
                                        "sugar": ingredients_updated.iter().find(|i| i.name == CookingIngredientName::Sugar).and_then(|object| Some(object.to_approx_unit_i18n().en)),
                                        "soy_sauce": ingredients_updated.iter().find(|i| i.name == CookingIngredientName::SoySauce).and_then(|object| Some(object.to_approx_unit_i18n().en)),
                                        "sesame": ingredients_updated.iter().find(|i| i.name == CookingIngredientName::Sesame).and_then(|object| Some(object.to_approx_unit_i18n().en)),
                                        "sesame_oil": ingredients_updated.iter().find(|i| i.name == CookingIngredientName::SesameOil).and_then(|object| Some(object.to_approx_unit_i18n().en)),
                                        "carrot": ingredients_updated.iter().find(|i| i.name == CookingIngredientName::Carrot).and_then(|object| Some(object.to_approx_unit_i18n().en)),
                                        "mayonnaise": ingredients_updated.iter().find(|i| i.name == CookingIngredientName::Mayonnaise).and_then(|object| Some(object.to_approx_unit_i18n().en)),
                                    })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                                    tts_script.ja = reg.render_template(&self.tts_script.ja, &json!({
                                        "salt": ingredients_updated.iter().find(|i| i.name == CookingIngredientName::Salt).and_then(|object| Some(object.to_approx_unit_i18n().ja)),
                                        "pepper": ingredients_updated.iter().find(|i| i.name == CookingIngredientName::Pepper).and_then(|object| Some(object.to_approx_unit_i18n().ja)),
                                        "sugar": ingredients_updated.iter().find(|i| i.name == CookingIngredientName::Sugar).and_then(|object| Some(object.to_approx_unit_i18n().ja)),
                                        "soy_sauce": ingredients_updated.iter().find(|i| i.name == CookingIngredientName::SoySauce).and_then(|object| Some(object.to_approx_unit_i18n().ja)),
                                        "sesame": ingredients_updated.iter().find(|i| i.name == CookingIngredientName::Sesame).and_then(|object| Some(object.to_approx_unit_i18n().ja)),
                                        "sesame_oil": ingredients_updated.iter().find(|i| i.name == CookingIngredientName::SesameOil).and_then(|object| Some(object.to_approx_unit_i18n().ja)),
                                        "carrot": ingredients_updated.iter().find(|i| i.name == CookingIngredientName::Carrot).and_then(|object| Some(object.to_approx_unit_i18n().ja)),
                                        "mayonnaise": ingredients_updated.iter().find(|i| i.name == CookingIngredientName::Mayonnaise).and_then(|object| Some(object.to_approx_unit_i18n().ja)),
                                    })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                                    tts_script.zh = reg.render_template(&self.tts_script.zh, &json!({
                                        "salt": ingredients_updated.iter().find(|i| i.name == CookingIngredientName::Salt).and_then(|object| Some(object.to_approx_unit_i18n().zh)),
                                        "pepper": ingredients_updated.iter().find(|i| i.name == CookingIngredientName::Pepper).and_then(|object| Some(object.to_approx_unit_i18n().zh)),
                                        "sugar": ingredients_updated.iter().find(|i| i.name == CookingIngredientName::Sugar).and_then(|object| Some(object.to_approx_unit_i18n().zh)),
                                        "soy_sauce": ingredients_updated.iter().find(|i| i.name == CookingIngredientName::SoySauce).and_then(|object| Some(object.to_approx_unit_i18n().zh)),
                                        "sesame": ingredients_updated.iter().find(|i| i.name == CookingIngredientName::Sesame).and_then(|object| Some(object.to_approx_unit_i18n().zh)),
                                        "sesame_oil": ingredients_updated.iter().find(|i| i.name == CookingIngredientName::SesameOil).and_then(|object| Some(object.to_approx_unit_i18n().zh)),
                                        "carrot": ingredients_updated.iter().find(|i| i.name == CookingIngredientName::Carrot).and_then(|object| Some(object.to_approx_unit_i18n().zh)),
                                        "mayonnaise": ingredients_updated.iter().find(|i| i.name == CookingIngredientName::Mayonnaise).and_then(|object| Some(object.to_approx_unit_i18n().zh)),
                                    })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                                    tts_script.ko = reg.render_template(&self.tts_script.ko, &json!({
                                        "salt": ingredients_updated.iter().find(|i| i.name == CookingIngredientName::Salt).and_then(|object| Some(object.to_approx_unit_i18n().ko)),
                                        "pepper": ingredients_updated.iter().find(|i| i.name == CookingIngredientName::Pepper).and_then(|object| Some(object.to_approx_unit_i18n().ko)),
                                        "sugar": ingredients_updated.iter().find(|i| i.name == CookingIngredientName::Sugar).and_then(|object| Some(object.to_approx_unit_i18n().ko)),
                                        "soy_sauce": ingredients_updated.iter().find(|i| i.name == CookingIngredientName::SoySauce).and_then(|object| Some(object.to_approx_unit_i18n().ko)),
                                        "sesame": ingredients_updated.iter().find(|i| i.name == CookingIngredientName::Sesame).and_then(|object| Some(object.to_approx_unit_i18n().ko)),
                                        "sesame_oil": ingredients_updated.iter().find(|i| i.name == CookingIngredientName::SesameOil).and_then(|object| Some(object.to_approx_unit_i18n().ko)),
                                        "carrot": ingredients_updated.iter().find(|i| i.name == CookingIngredientName::Carrot).and_then(|object| Some(object.to_approx_unit_i18n().ko)),
                                        "mayonnaise": ingredients_updated.iter().find(|i| i.name == CookingIngredientName::Mayonnaise).and_then(|object| Some(object.to_approx_unit_i18n().ko)),
                                    })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                                    return Ok(SmartSpeakerTaskResult::with_tts_and_revision(
                                        self.get_action_trigger_type().to_task_type(),
                                        SmartSpeakerTaskResultCode::StepSuccess,
                                        tts_script,
                                        Box::new(CookingRevision::new(revision_entities_updated)),
                                    ));
                                }
                            }
                        }
                    }
                    CookingActionDetail::ExplainMutableTime(criteria) => {
                        match &self.current_revision {
                            None => {
                                tts_script.en = reg.render_template(&self.tts_script.en, &json!({
                                    "time": criteria.to_human_time().en
                                })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                                tts_script.ja = reg.render_template(&self.tts_script.ja, &json!({
                                    "time": criteria.to_human_time().ja
                                })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                                tts_script.zh = reg.render_template(&self.tts_script.zh, &json!({
                                    "time": criteria.to_human_time().zh
                                })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                                tts_script.ko = reg.render_template(&self.tts_script.ko, &json!({
                                    "time": criteria.to_human_time().ko
                                })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                            }
                            Some(rev) => {
                                let mut revision_entities_updated = vec![];
                                self.current_revision.clone().and_then(|mut r| {
                                    r.entities.pop().and_then(
                                        |entity| {
                                            let _ = criteria.calc_time_by_revision(&entity).and_then(|time| {
                                                tts_script.en = reg.render_template(&self.tts_script.en, &json!({
                                        "time": time.to_human_time().en
                                    })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                                                tts_script.ja = reg.render_template(&self.tts_script.ja, &json!({
                                        "time": time.to_human_time().ja
                                    })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                                                tts_script.zh = reg.render_template(&self.tts_script.zh, &json!({
                                        "time": time.to_human_time().zh
                                    })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                                                tts_script.ko = reg.render_template(&self.tts_script.ko, &json!({
                                        "time": time.to_human_time().ko
                                    })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                                                None::<i32>
                                            });
                                            None::<CookingRevision>
                                        });
                                    revision_entities_updated = r.entities;
                                    None::<CookingRevision>
                                });
                                return Ok(SmartSpeakerTaskResult::with_tts_and_revision(
                                    self.get_action_trigger_type().to_task_type(),
                                    SmartSpeakerTaskResultCode::StepSuccess,
                                    tts_script,
                                    Box::new(CookingRevision::new(revision_entities_updated)),
                                ));
                            }
                        }
                    }
                    _ => {}
                }
                self.current_revision.as_ref().map_or_else(
                    || Ok(SmartSpeakerTaskResult::with_tts(
                        self.get_action_trigger_type().to_task_type(),
                        SmartSpeakerTaskResultCode::StepSuccess,
                        tts_script.clone(),
                    )),
                    |rev| {
                        Ok(SmartSpeakerTaskResult::with_tts_and_revision(
                            self.get_action_trigger_type().to_task_type(),
                            SmartSpeakerTaskResultCode::StepSuccess,
                            tts_script.clone(),
                            Box::new(rev.clone()),
                        ))
                    })
            },
            _ => {
                self.current_revision.as_ref().map_or_else(
                   ||  Ok(SmartSpeakerTaskResult::with_tts(
                       self.get_action_trigger_type().to_task_type(),
                        SmartSpeakerTaskResultCode::StepFailed,
                        tts_script.clone(),
                    )),
                    |rev| {
                        Ok(SmartSpeakerTaskResult::with_tts_and_revision(
                            self.get_action_trigger_type().to_task_type(),
                            SmartSpeakerTaskResultCode::StepFailed,
                            tts_script.clone(),
                            Box::new(rev.clone()),
                        ))
                    })
            }
        }
    }

    fn feed(&mut self, content: Box<dyn Content>, revision: Option<Box<dyn Revision>>) -> Result<()> {
        self.check_cancelled(&content)?;
        self.check_request_repeat(&content)?;
        if let Some(content) = content.as_any().downcast_ref::<IntentContent>() {
            self.current_content = Some(content.clone());
        }
        match revision {
            None => {}
            Some(rev) => {
                if let Some(r) = rev.as_any().downcast_ref::<CookingRevision>() {
                    self.current_revision = Some(r.clone());
                }
            }
        }
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn ActionExecutable> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_action_trigger_type(&self) -> ActionTriggerType {
        ActionTriggerType::Confirm
    }

    fn get_cancelled(&self) -> bool {
        self.cancelled
    }

    fn set_cancelled(&mut self) -> Result<()> {
        self.cancelled = true;
        Ok(())
    }

    fn get_repeated(&self) -> bool {
        self.repeat_requested
    }

    fn set_repeated(&mut self) -> Result<()> {
        self.repeat_requested = true;
        Ok(())
    }

    fn expose_tts_script(&self) -> Result<SmartSpeakerI18nText> {
        Ok(self.tts_script.clone())
    }

    fn try_expose_vision_actions(&self) -> Result<Vec<VisionAction>> {
        Err(anyhow!("Not a vision action"))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct VisionBasedIngredientMeasureAction {
    pub(crate) ingredients: Vec<CookingIngredient>,
    pub(crate) detail: CookingActionDetail,
    pub(crate) vision_action: VisionAction,
    pub(crate) tts_script: SmartSpeakerI18nText,
    pub(crate) current_content: Option<VisionContent>,
    pub(crate) current_revision: Option<CookingRevision>,
    cancelled: bool,
    repeat_requested: bool,
}

impl VisionBasedIngredientMeasureAction {
    pub(crate) fn new(ingredients: Vec<CookingIngredient>, detail: CookingActionDetail, vision_action: VisionAction, text: SmartSpeakerI18nText) -> Self {
        VisionBasedIngredientMeasureAction {
            ingredients,
            detail,
            vision_action,
            tts_script: text,
            current_content: None,
            current_revision: None,
            cancelled: false,
            repeat_requested: false,
        }
    }

    pub(crate) fn handle_vision_contents(&self, contents: &Vec<VisionObject>) -> Result<SmartSpeakerTaskResult> {
        let mut revisions: Vec<CookingRevisionEntity> = vec![];
        let mut reg = Handlebars::new();
        let mut tts_script = self.tts_script.clone();
        self.current_revision.clone().and_then(|rev| {
            for entity in rev.entities {
                revisions.push(entity);
            }
            None::<CookingRevision>
        });
        if contents.len() == 0 {
            return Ok(SmartSpeakerTaskResult::with_tts(
                self.get_action_trigger_type().to_task_type(),
                SmartSpeakerTaskResultCode::StepFailed,
                self.tts_script.clone(),
            ))
        }

        match self.detail {
            CookingActionDetail::MeasureIngredientSize => {
                let first = contents.get(0).unwrap();
                let last_revision = revisions.last();
                match first.object_type {
                    DetectableObject::Carrot => {
                        let target = self.ingredients.iter().find(|i| i.name == CookingIngredientName::Carrot).unwrap();
                        let weight_approx = target.name.get_weight_per_perimeter(first.size.perimeter);
                        let mut diff = weight_approx.sub(target.unit).unwrap();
                        // match last_revision {
                        //     None => {}
                        //     Some(rev) => {
                        //         match rev.property {
                        //             CookingRevisionEntityProperty::Add(ingredient) => {
                        //                 if ingredient.name == CookingIngredientName::Carrot {
                        //                     let previous_weight = ingredient.unit.get_value() + 1000.0;
                        //                     if weight_approx.get_value() > previous_weight {
                        //                         let spread = weight_approx.get_value() / previous_weight;
                        //                         let pieces_approx = (previous_weight * spread) / target.name.get_weight_per_perimeter();
                        //                         let approx_weight_per_piece = previous_weight / pieces_approx;
                        //                     }
                        //                 }
                        //             }
                        //             CookingRevisionEntityProperty::Sub(ingredient) => {
                        //                 if ingredient.name == CookingIngredientName::Carrot {
                        //                     let previous_weight = 1000.0 - ingredient.unit.get_value();
                        //                     if weight_approx.get_value() > previous_weight {
                        //                         let spread = weight_approx.get_value() / previous_weight;
                        //                         let pieces_approx = (previous_weight * spread) / target.name.get_weight_per_perimeter(target.name.get_perimeter_per_piece());
                        //                     }
                        //                 }
                        //             }
                        //         }
                        //     }
                        // }
                        if diff.get_value().is_sign_positive() {
                            revisions.push(CookingRevisionEntity::new(
                                0,
                                CookingRevisionEntityProperty::Add(CookingIngredient {
                                    name: CookingIngredientName::Carrot,
                                    unit: diff.abs(),
                                })));
                            tts_script.ko = reg.render_template(&self.tts_script.ko, &json!({
                                "measure_result": "레시피에서 요구하는 양보다 더 많은 것 같네요."
                            })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                                tts_script.en = reg.render_template(&self.tts_script.en, &json!({
                                "measure_result": "It seems to be more than the amount required by the recipe."
                            })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                                tts_script.ja = reg.render_template(&self.tts_script.ja, &json!({
                                "measure_result": "レシピで必要な量よりも多いようです。"
                            })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                                tts_script.zh = reg.render_template(&self.tts_script.zh, &json!({
                                "measure_result": "看起来比食谱所需的量多。"
                            })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                        } else {
                            revisions.push(CookingRevisionEntity::new(
                                0,
                                CookingRevisionEntityProperty::Sub(CookingIngredient {
                                    name: CookingIngredientName::Carrot,
                                    unit: diff.abs(),
                                })));
                            tts_script.ko = reg.render_template(&self.tts_script.ko, &json!({
                                "measure_result": "레시피에서 요구하는 양보다 더 적은 것 같네요."
                            })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                                tts_script.en = reg.render_template(&self.tts_script.en, &json!({
                                "measure_result": "It seems to be less than the amount required by the recipe."
                            })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                                tts_script.ja = reg.render_template(&self.tts_script.ja, &json!({
                                "measure_result": "レシピで必要な量よりも少ないようです。"
                            })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                                tts_script.zh = reg.render_template(&self.tts_script.zh, &json!({
                                "measure_result": "看起来比食谱所需的量少。"
                            })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                        }
                        if diff.get_value() <= (target.unit.get_value() * 0.05) {
                            tts_script.ko = reg.render_template(&self.tts_script.ko, &json!({
                                "measure_result": "레시피에서 요구하는 양과 비슷한 것 같네요."
                            })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                            tts_script.en = reg.render_template(&self.tts_script.en, &json!({
                                "measure_result": "It seems to be similar to the amount required by the recipe."
                            })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                            tts_script.ja = reg.render_template(&self.tts_script.ja, &json!({
                                "measure_result": "レシピで必要な量と似ているようです。"
                            })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                            tts_script.zh = reg.render_template(&self.tts_script.zh, &json!({
                                "measure_result": "看起来与食谱所需的量相似。"
                            })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                        }
                    }
                    DetectableObject::Potato => {
                        let target = self.ingredients.iter().find(|i| i.name == CookingIngredientName::Potato).unwrap();
                        let weight_approx = target.name.get_weight_per_perimeter(first.size.perimeter);
                        let diff = weight_approx.sub(target.unit).unwrap();
                        if diff.get_value().is_sign_positive() {
                            revisions.push(CookingRevisionEntity::new(
                                0,
                                CookingRevisionEntityProperty::Add(CookingIngredient {
                                    name: CookingIngredientName::Potato,
                                    unit: diff.abs(),
                                })));
                            tts_script.ko = reg.render_template(&self.tts_script.ko, &json!({
                                "measure_result": "레시피에서 요구하는 양보다 더 많은 것 같네요."
                            })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                            tts_script.en = reg.render_template(&self.tts_script.en, &json!({
                                "measure_result": "It seems to be more than the amount required by the recipe."
                            })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                            tts_script.ja = reg.render_template(&self.tts_script.ja, &json!({
                                "measure_result": "レシピで必要な量よりも多いようです。"
                            })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                            tts_script.zh = reg.render_template(&self.tts_script.zh, &json!({
                                "measure_result": "看起来比食谱所需的量多。"
                            })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                        } else {
                            revisions.push(CookingRevisionEntity::new(
                                0,
                                CookingRevisionEntityProperty::Sub(CookingIngredient {
                                    name: CookingIngredientName::Potato,
                                    unit: diff.abs(),
                                })));
                            tts_script.ko = reg.render_template(&self.tts_script.ko, &json!({
                                "measure_result": "레시피에서 요구하는 양보다 더 적은 것 같네요."
                            })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                            tts_script.en = reg.render_template(&self.tts_script.en, &json!({
                                "measure_result": "It seems to be less than the amount required by the recipe."
                            })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                            tts_script.ja = reg.render_template(&self.tts_script.ja, &json!({
                                "measure_result": "レシピで必要な量よりも少ないようです。"
                            })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                            tts_script.zh = reg.render_template(&self.tts_script.zh, &json!({
                                "measure_result": "看起来比食谱所需的量少。"
                            })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                        }
                        if diff.get_value() <= (target.unit.get_value() * 0.05) {
                            tts_script.ko = reg.render_template(&self.tts_script.ko, &json!({
                                "measure_result": "레시피에서 요구하는 양과 비슷한 것 같네요."
                            })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                            tts_script.en = reg.render_template(&self.tts_script.en, &json!({
                                "measure_result": "It seems to be similar to the amount required by the recipe."
                            })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                            tts_script.ja = reg.render_template(&self.tts_script.ja, &json!({
                                "measure_result": "レシピで必要な量と似ているようです。"
                            })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                            tts_script.zh = reg.render_template(&self.tts_script.zh, &json!({
                                "measure_result": "看起来与食谱所需的量相似。"
                            })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        return Ok(SmartSpeakerTaskResult::with_tts_and_revision(
            self.get_action_trigger_type().to_task_type(),
            SmartSpeakerTaskResultCode::StepSuccess,
            tts_script.clone(),
            Box::new(CookingRevision::new(revisions)),
        ))
    }
}

impl ActionExecutable for VisionBasedIngredientMeasureAction {
    fn execute(&self) -> Result<SmartSpeakerTaskResult> {
        if self.has_cancelled() {
            return Ok(SmartSpeakerTaskResult::new(
                self.get_action_trigger_type().to_task_type(),
                SmartSpeakerTaskResultCode::Cancelled));
        }
        if self.has_request_repeat() {
            return Ok(SmartSpeakerTaskResult::new(
                self.get_action_trigger_type().to_task_type(),
                SmartSpeakerTaskResultCode::RepeatPrevious));
        }
        return match &self.current_content {
            None => {
                Ok(SmartSpeakerTaskResult::with_tts(
                    self.get_action_trigger_type().to_task_type(),
                    SmartSpeakerTaskResultCode::StepFailed,
                    self.tts_script.clone(),
                ))
            }
            Some(content) => {
                match &content.action {
                    VisionAction::None => {
                        Ok(SmartSpeakerTaskResult::with_tts(
                            self.get_action_trigger_type().to_task_type(),
                            SmartSpeakerTaskResultCode::StepSuccess,
                            self.tts_script.clone(),
                        ))
                    }
                    VisionAction::ObjectDetection(detectable) => {
                        self.handle_vision_contents(
                            &content.entities.iter().map(|c| c.as_any().downcast_ref::<VisionObject>().unwrap().clone()).collect::<Vec<VisionObject>>())
                    }
                }
            }
        }
    }

    fn feed(&mut self, content: Box<dyn Content>, revision: Option<Box<dyn Revision>>) -> Result<()> {
        self.check_cancelled(&content)?;
        self.check_request_repeat(&content)?;
        if let Some(content) = content.as_any().downcast_ref::<VisionContent>() {
            self.current_content = Some(content.clone());
        }
        match revision {
            None => {}
            Some(rev) => {
                if let Some(r) = rev.as_any().downcast_ref::<CookingRevision>() {
                    self.current_revision = Some(r.clone());
                }
            }
        }
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn ActionExecutable> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_action_trigger_type(&self) -> ActionTriggerType {
        ActionTriggerType::Vision(vec![self.vision_action.clone()])
    }

    fn get_cancelled(&self) -> bool {
        self.cancelled
    }

    fn set_cancelled(&mut self) -> Result<()> {
        self.cancelled = true;
        Ok(())
    }

    fn get_repeated(&self) -> bool {
        self.repeat_requested
    }

    fn set_repeated(&mut self) -> Result<()> {
        self.repeat_requested = true;
        Ok(())
    }

    fn expose_tts_script(&self) -> Result<SmartSpeakerI18nText> {
        Ok(self.tts_script.clone())
    }

    fn try_expose_vision_actions(&self) -> Result<Vec<VisionAction>> {
        Ok(vec![self.vision_action.clone()])
    }
}

pub(crate) struct CookingStepBuilder {
    vision: bool
}

impl CookingStepBuilder {
    pub(crate) fn new(vision: bool) -> Self {
        CookingStepBuilder {
            vision
        }
    }

    fn build_carrot_salad(&self, menu: &IntentCookingMenu, steps: &mut Vec<Box<dyn ActionExecutable>>) {
        steps.push(
            Box::new(ExplainRecipeAction::new(
                vec![],
                CookingActionDetail::None,
                SmartSpeakerI18nText::new()
                    .ko("먼저 당근을 준비합니다.")
                    .en("First, prepare the carrots.")
                    .ja("まず人参　を用意します。")
                    .zh("首先准备胡萝卜。")
            )));
        if self.vision {
            steps.push(
                Box::new(ExplainRecipeAction::new(
                    vec![],
                    CookingActionDetail::None,
                    SmartSpeakerI18nText::new()
                        .ko("도마 위에 당근이 준비되었다면, 저도 볼 수 있게 손을 치워주실 수 있나요? 준비가 되면 알려주세요.")
                        .en("If the carrots are ready on the chopping board, can you move your hands back so that I can see? Let me know when you are ready.")
                        .ja("まな板の上に人参が用意できたら、私にも見えるように手をどけてくれますか？ 準備ができたら教えてください。")
                        .zh("如果胡萝卜准备好了，你能把手拿开让我看看吗？ 准备好后请告诉我。")
                ))
            );
            steps.push(
                Box::new(ExplainRecipeAction::new(
                    vec![],
                    CookingActionDetail::None,
                    SmartSpeakerI18nText::new()
                        .ko("감사합니다.")
                        .en("Thank you.")
                        .ja("ありがとうございます。")
                        .zh("谢谢。")
                ))
            );
            steps.push(
                Box::new(VisionBasedIngredientMeasureAction::new(
                    vec![menu.to_ingredient().iter().find(|i| i.name == CookingIngredientName::Carrot).unwrap().clone()],
                    CookingActionDetail::MeasureIngredientSize,
                    VisionAction::ObjectDetection(DetectionDetail::new(
                        DetectionMode::Aruco,
                        DetectableObject::Carrot,
                        true,
                    )),
                    SmartSpeakerI18nText::new()
                        .ko("{{measure_result}} 이후의 설명에 참고하도록 하겠습니다.")
                        .en("{{measure_result}} I'll keep that in mind for the rest of the instructions.")
                        .ja("{{measure_result}} 残りの説明のために覚えておきます。")
                        .zh("{{measure_result}} 我会记住剩下的说明。")
                ))
            );
        }
        steps.push(
            Box::new(ExplainRecipeAction::new(
                vec![],
                CookingActionDetail::None,
                SmartSpeakerI18nText::new()
                    .ko("계속해서 당근을 먹기 좋은 크기로 썰어주세요.")
                    .en("Please continue to cut the carrots into bite-sized pieces.")
                    .ja("続いて、人参を食べやすい大きさに切ってください。")
                    .zh("请继续把胡萝卜切成一口大小。")
            )));
        if self.vision {
            steps.push(
                Box::new(ExplainRecipeAction::new(
                    vec![],
                    CookingActionDetail::None,
                    SmartSpeakerI18nText::new()
                        .ko("당근을 어떻게 자르셨는지 저도 볼 수 있게 한 조각만 보여주실 수 있나요? 준비가 되면 당근을 바라본 채 알려주세요.")
                        .en("Can you show me just one piece so I can see how you cut the carrot? When you're ready, look at the carrot and let me know.")
                        .ja("人参をどのように切ったか、私にも1こだけを見せてもらえますか？ 準備ができたら、人参を見て教えてください。")
                        .zh("你能给我看一块胡萝卜吗？ 准备好后，请看着胡萝卜告诉我。")
                ))
            );
            steps.push(
                Box::new(ExplainRecipeAction::new(
                    vec![],
                    CookingActionDetail::None,
                    SmartSpeakerI18nText::new()
                        .ko("감사합니다.")
                        .en("Thank you.")
                        .ja("ありがとうございます。")
                        .zh("谢谢。")
                ))
            );
            steps.push(
                Box::new(VisionBasedIngredientMeasureAction::new(
                    vec![CookingIngredient::new(
                        CookingIngredientName::Carrot,
                        CookingIngredientAmount::MilliGram(100))],
                    CookingActionDetail::MeasureIngredientSize,
                    VisionAction::ObjectDetection(DetectionDetail::new(
                        DetectionMode::Aruco,
                        DetectableObject::Carrot,
                        true,
                    )),
                    SmartSpeakerI18nText::new()
                        .ko("{{measure_result}} 이후의 설명에 참고하도록 하겠습니다.")
                        .en("{{measure_result}} I'll keep that in mind for the rest of the instructions.")
                        .ja("{{measure_result}} 残りの説明のために覚えておきます。")
                        .zh("{{measure_result}} 我会记住剩下的说明。")
                ))
            );
        }
        steps.push(
            Box::new(ExplainRecipeAction::new(
                vec![menu.to_ingredient().iter().find(|i| i.name == CookingIngredientName::Carrot).unwrap().clone()],
                CookingActionDetail::ExplainMutableTime(
                    CookingIngredientTime::new(
                        CookingIngredient::new(
                            CookingIngredientName::Carrot,
                            CookingIngredientAmount::MilliGram(150)),
                        80)),
                SmartSpeakerI18nText::new()
                    .ko("손질한 당근을 끓는 물에 약 {{time}}간 삶아주세요.")
                    .en("Boil the carrots in boiling water for about {{time}}.")
                    .ja("人参を沸いた水に、。。。約、。。。{{time}}間、。。。茹でます。")
                    .zh("把胡萝卜放在沸水里煮、。。。约、。。。{{time}}钟。")
            )));
        steps.push(
            Box::new(ExplainRecipeAction::new(
                menu.to_ingredient(),
                CookingActionDetail::ExplainMutableIngredient(
                    CookingIngredientLinkComponent::new(
                        menu.to_ingredient().iter().find(|i| i.name == CookingIngredientName::Carrot).unwrap().clone(),
                        menu.to_ingredient().iter().filter(|ing| matches!(ing.name, CookingIngredientName::Salt|CookingIngredientName::Pepper|CookingIngredientName::SesameOil)).map(|ing| ing.clone()).collect::<Vec<CookingIngredient>>()
                    )),
                SmartSpeakerI18nText::new()
                    .ko("삶은 당근을 보울에 담아 소금 {{salt}},    후추 {{pepper}},    참기름 {{sesame_oil}}을 넣고 섞어주세요.")
                    .en("Put the boiled carrots in a bowl and add {{salt}} of salt,    {{pepper}} of pepper,    and {{sesame_oil}} of sesame oil.")
                    .ja("茹でた人参をボウルに入れて塩　{{salt}}、　　　コショウ　{{pepper}}、　　　ごま油　{{sesame_oil}}　　　を入れて混ぜます。")
                    .zh("把煮好的胡萝卜放在碗里，加{{salt}}的盐，   {{pepper}}的胡椒粉，   {{sesame_oil}}的芝麻油，并搅拌。")
            )));
    }

    fn build_potato_salad(&self, menu: &IntentCookingMenu, steps: &mut Vec<Box<dyn ActionExecutable>>) {
        steps.push(
            Box::new(ExplainRecipeAction::new(
                vec![],
                CookingActionDetail::None,
                SmartSpeakerI18nText::new()
                    .ko("먼저 감자를 준비합니다.")
                    .en("First, prepare the potatoes.")
                    .ja("まずじゃがいも　を用意します。")
                    .zh("首先准备土豆。")
            )));
        if self.vision {
            steps.push(
                Box::new(ExplainRecipeAction::new(
                    vec![],
                    CookingActionDetail::None,
                    SmartSpeakerI18nText::new()
                        .ko("도마 위에 감자가 준비되었다면, 저도 볼 수 있게 손을 치워주실 수 있나요? 준비가 되면 알려주세요.")
                        .en("If the potatoes are ready on the chopping board, can you move your hands back so that I can see? Let me know when you are ready.")
                        .ja("まな板の上にじゃがいもが用意できたら、私にも見えるように手をどけてくれますか？ 準備ができたら教えてください。")
                        .zh("如果土豆准备好了，你能把手拿开让我看看吗？ 准备好后请告诉我。")
                ))
            );
            steps.push(
                Box::new(ExplainRecipeAction::new(
                    vec![],
                    CookingActionDetail::None,
                    SmartSpeakerI18nText::new()
                        .ko("감사합니다.")
                        .en("Thank you.")
                        .ja("ありがとうございます。")
                        .zh("谢谢。")
                ))
            );
            steps.push(
                Box::new(VisionBasedIngredientMeasureAction::new(
                    vec![menu.to_ingredient().iter().find(|i| i.name == CookingIngredientName::Potato).unwrap().clone()],
                    CookingActionDetail::MeasureIngredientSize,
                    VisionAction::ObjectDetection(DetectionDetail::new(
                        DetectionMode::Aruco,
                        DetectableObject::Potato,
                        true,
                    )),
                    SmartSpeakerI18nText::new()
                        .ko("{{measure_result}} 이후의 설명에 참고하도록 하겠습니다.")
                        .en("{{measure_result}} I'll keep that in mind for the rest of the instructions.")
                        .ja("{{measure_result}} 残りの説明のために覚えておきます。")
                        .zh("{{measure_result}} 我会记住剩下的说明。")
                ))
            );
        }
        steps.push(
            Box::new(ExplainRecipeAction::new(
                vec![],
                CookingActionDetail::None,
                SmartSpeakerI18nText::new()
                    .ko("계속해서 감자를 먹기 좋은 크기로 썰어주세요.")
                    .en("Please continue to cut the potatoes into bite-sized pieces.")
                    .ja("続いて、じゃがいもを食べやすい大きさに切ってください。")
                    .zh("请继续把土豆切成一口大小。")
            )));
        if self.vision {
            steps.push(
                Box::new(ExplainRecipeAction::new(
                    vec![],
                    CookingActionDetail::None,
                    SmartSpeakerI18nText::new()
                        .ko("감자를 어떻게 자르셨는지 저도 볼 수 있게 한 조각만 보여주실 수 있나요? 준비가 되면 감자를 바라본 채 알려주세요.")
                        .en("Can you show me just one piece so I can see how you cut the potato? When you're ready, look at the potato and let me know.")
                        .ja("じゃがいもをどのように切ったか、私にも1こだけを見せてもらえますか？ 準備ができたら、じゃがいもを見て教えてください。")
                        .zh("你能给我看一块土豆吗？ 准备好后，请看着土豆告诉我。")
                ))
            );
            steps.push(
                Box::new(ExplainRecipeAction::new(
                    vec![],
                    CookingActionDetail::None,
                    SmartSpeakerI18nText::new()
                        .ko("감사합니다.")
                        .en("Thank you.")
                        .ja("ありがとうございます。")
                        .zh("谢谢。")
                ))
            );
            steps.push(
                Box::new(VisionBasedIngredientMeasureAction::new(
                    vec![CookingIngredient::new(
                        CookingIngredientName::Potato,
                        CookingIngredientAmount::MilliGram(150))],
                    CookingActionDetail::MeasureIngredientSize,
                    VisionAction::ObjectDetection(DetectionDetail::new(
                        DetectionMode ::Aruco,
                        DetectableObject::Potato,
                        true,
                    )),
                    SmartSpeakerI18nText::new()
                        .ko("{{measure_result}} 이후의 설명에 참고하도록 하겠습니다.")
                        .en("{{measure_result}} I'll keep that in mind for the rest of the instructions.")
                        .ja("{{measure_result}} 残りの説明のために覚えておきます。")
                        .zh("{{measure_result}} 我会记住剩下的说明。")
                ))
            );
        }
        steps.push(
            Box::new(ExplainRecipeAction::new(
                vec![menu.to_ingredient().iter().find(|i| i.name == CookingIngredientName::Potato).unwrap().clone()],
                CookingActionDetail::ExplainMutableTime(
                    CookingIngredientTime::new(
                        CookingIngredient::new(
                            CookingIngredientName::Potato,
                            CookingIngredientAmount::MilliGram(100)),
                        80)),
                SmartSpeakerI18nText::new()
                    .ko("손질한 감자를 끓는 물에 약 {{time}}간 삶아주세요.")
                    .en("Boil the potatoes in boiling water for about {{time}}.")
                    .ja("じゃがいもを沸いた水に、。。。約、。。。{{time}}間、。。。茹でます。")
                    .zh("把土豆放在沸水里煮、。。。约、。。。{{time}}钟。")
            )));
        steps.push(
            Box::new(ExplainRecipeAction::new(
                menu.to_ingredient(),
                CookingActionDetail::ExplainMutableIngredient(
                    CookingIngredientLinkComponent::new(
                        menu.to_ingredient().iter().find(|i| i.name == CookingIngredientName::Potato).unwrap().clone(),
                        menu.to_ingredient().iter().filter(|ing| matches!(ing.name, CookingIngredientName::Salt|CookingIngredientName::Pepper|CookingIngredientName::Mayonnaise)).map(|ing| ing.clone()).collect::<Vec<CookingIngredient>>()
                    )),
                SmartSpeakerI18nText::new()
                    .ko("삶은 감자를 보울에 담아 소금 {{salt}},    후추 {{pepper}},    마요네즈 {{mayonnaise}}을 넣고 섞어주세요.")
                    .en("Put the boiled potatoes in a bowl and add {{salt}} of salt,    {{pepper}} of pepper,    and {{mayonnaise}} of mayonnaise.")
                    .ja("茹でたじゃがいもをボウルに入れて塩　{{salt}}、　　　コショウ　{{pepper}}、　　　マヨネーズ　{{mayonnaise}}　　　を入れて混ぜます。")
                    .zh("把煮好的土豆放在碗里，加{{salt}}的盐，   {{pepper}}的胡椒粉，   {{mayonnaise}}的蛋黄酱，并搅拌。")
            )));
    }

    pub(crate) fn build(&self, menu: IntentCookingMenu) -> Vec<Box<dyn ActionExecutable>> {
        let mut steps: Vec<Box<dyn ActionExecutable>> = vec![];
        steps.push(
            Box::new(ExplainRecipeAction::new(
                vec![],
                CookingActionDetail::None,
                SmartSpeakerI18nText::new()
                    .ko(&format!("{} 요리를 시작합니다. 지금부터는 '헤이 링고' 로 저를 부르지 않아도 됩니다. 다음 작업으로 넘어가려면 '오케이' 또는 '다음' 과 같은 대답으로 알려주세요.", menu.to_i18n().ko))
                    .en(&format!("Let's start {} cooking. You don't have to call me 'Hey Ringo' from now on. Please let me know if you want to proceed to the next step by answering 'OK' or 'Next'.", menu.to_i18n().en))
                    .ja(&format!("{} 料理を始めます。 これからは「ヘイ、リンゴ」と呼ばなくてもいいです。 次の作業に進みたい場合は、「OK」や「次」などの答えで教えてください。", menu.to_i18n().ja))
                    .zh(&format!("开始{}烹饪。 从现在开始，你不必叫我“嘿，拎郭”。 如果你想继续下一步，请回答　“可以”　或　“接下来”。", menu.to_i18n().zh))
            )));
        steps.push(
            Box::new(ExplainRecipeAction::new(
                menu.to_ingredient(),
                CookingActionDetail::ExplainNonMutableIngredient,
                SmartSpeakerI18nText::new()
                    .ko("요리 재료 설명을 시작합니다. {{additional_explain}} 가 필요합니다. 다시 한 번 들으시려면 '다시 알려 줘' 라고 말씀해주세요.")
                    .en("Let's start explaining ingredients. {{additional_explain}} is required. If you want to hear it again, please say 'tell me again'.")
                    .ja("食材の説明を始めます。{{additional_explain}} が必要です。もう一度聞きたい場合は、「もう一度教えて」と言ってください。")
                    .zh("让我们开始解释食材。{{additional_explain}} 是必需的。如果你想再听一遍，请说“再告诉我一遍”。")
            )));
        match menu {
            IntentCookingMenu::CarrotSalad => {
                self.build_carrot_salad(&menu, &mut steps);
            }
            IntentCookingMenu::PotatoSalad => {
                self.build_potato_salad(&menu, &mut steps);
            }
        }

        steps.push(
            Box::new(ExplainRecipeAction::new(
                vec![],
                CookingActionDetail::None,
                SmartSpeakerI18nText::new()
                    .ko("완성된 요리를 보기 좋게 접시에 담아주세요.")
                    .en("Put the finished dish on a plate.")
                    .ja("完成した料理をきれいにお皿に盛り付けます。")
                    .zh("把做好的菜放在盘子里。")
            ))
        );
        steps.push(
            Box::new(ExplainRecipeAction::new(
                vec![],
                CookingActionDetail::None,
                SmartSpeakerI18nText::new()
                    .ko("완성입니다. 맛있게 드세요.")
                    .en("It's done. Bon appetit.")
                    .ja("完成です。おいしく召し上がってください。")
                    .zh("完成了。请享用。")
            ))
        );

        steps
    }
}
