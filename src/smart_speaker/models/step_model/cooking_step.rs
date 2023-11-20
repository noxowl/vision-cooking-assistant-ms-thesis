use std::fmt::{Debug};
use anyhow::{anyhow, Result};
use handlebars::Handlebars;
use serde_json::json;
use crate::smart_speaker::models::intent_model::IntentCookingMenu;
use crate::smart_speaker::models::step_model::generic_step::{ActionExecutable, ActionTriggerType};
use crate::smart_speaker::models::task_model::{SmartSpeakerTaskResult, SmartSpeakerTaskResultCode};
use crate::smart_speaker::models::vision_model::{DetectableObject, VisionAction, VisionObject};
use crate::smart_speaker::models::message_model::*;
use crate::smart_speaker::models::revision_model::cooking_revision::{CookingRevision, CookingRevisionEntity};
use crate::smart_speaker::models::revision_model::Revision;
use crate::smart_speaker::models::task_model::cooking_task::{CookingIngredient, CookingIngredientAmount, CookingIngredientLinkComponent, CookingIngredientName, CookingIngredientTime, SmartSpeakerMaterialProperty};

#[derive(Debug, Clone)]
pub(crate) enum CookingActionDetail {
    None,
    ExplainNonMutableIngredient,
    ExplainMutableIngredient(CookingIngredientLinkComponent),
    ExplainMutableTime(CookingIngredientTime),
    MeasureWholeIngredient,
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
                SmartSpeakerTaskResultCode::Cancelled));
        }
        if self.has_request_repeat() {
            return Ok(SmartSpeakerTaskResult::new(
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
                            }).collect::<Vec<String>>().join(". ")
                        })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                        tts_script.ja = reg.render_template(&self.tts_script.ja, &json!({
                            "additional_explain": self.ingredients.iter()
                            .map(|i| {
                                if i.name.to_material_property() == SmartSpeakerMaterialProperty::Solid {
                                    format!("{} {}", i.name.to_i18n().ja, i.to_approx_unit_i18n().ja)
                                } else {
                                    format!("{}", i.name.to_i18n().ja)
                                }
                            }).collect::<Vec<String>>().join("、")
                        })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                        tts_script.zh = reg.render_template(&self.tts_script.zh, &json!({
                            "additional_explain": self.ingredients.iter()
                            .map(|i| {
                                if i.name.to_material_property() == SmartSpeakerMaterialProperty::Solid {
                                    format!("{} {}", i.name.to_i18n().zh, i.to_approx_unit_i18n().zh)
                                } else {
                                    format!("{}", i.name.to_i18n().zh)
                                }
                            }).collect::<Vec<String>>().join("、")
                        })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                        tts_script.ko = reg.render_template(&self.tts_script.ko, &json!({
                            "additional_explain": self.ingredients.iter()
                            .map(|i| {
                                if i.name.to_material_property() == SmartSpeakerMaterialProperty::Solid {
                                    format!("{} {}", i.name.to_i18n().ko, i.to_approx_unit_i18n().ko)
                                } else {
                                    format!("{}", i.name.to_i18n().ko)
                                }
                            }).collect::<Vec<String>>().join(". ")
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
                                })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                                tts_script.ja = reg.render_template(&self.tts_script.ja, &json!({
                                    "salt": self.ingredients.iter().find(|i| i.name == CookingIngredientName::Salt).and_then(|object| Some(object.to_approx_unit_i18n().ja)),
                                    "pepper": self.ingredients.iter().find(|i| i.name == CookingIngredientName::Pepper).and_then(|object| Some(object.to_approx_unit_i18n().ja)),
                                    "sugar": self.ingredients.iter().find(|i| i.name == CookingIngredientName::Sugar).and_then(|object| Some(object.to_approx_unit_i18n().ja)),
                                    "soy_sauce": self.ingredients.iter().find(|i| i.name == CookingIngredientName::SoySauce).and_then(|object| Some(object.to_approx_unit_i18n().ja)),
                                    "sesame": self.ingredients.iter().find(|i| i.name == CookingIngredientName::Sesame).and_then(|object| Some(object.to_approx_unit_i18n().ja)),
                                    "sesame_oil": self.ingredients.iter().find(|i| i.name == CookingIngredientName::SesameOil).and_then(|object| Some(object.to_approx_unit_i18n().ja)),
                                    "carrot": self.ingredients.iter().find(|i| i.name == CookingIngredientName::Carrot).and_then(|object| Some(object.to_approx_unit_i18n().ja)),
                                })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                                tts_script.zh = reg.render_template(&self.tts_script.zh, &json!({
                                    "salt": self.ingredients.iter().find(|i| i.name == CookingIngredientName::Salt).and_then(|object| Some(object.to_approx_unit_i18n().zh)),
                                    "pepper": self.ingredients.iter().find(|i| i.name == CookingIngredientName::Pepper).and_then(|object| Some(object.to_approx_unit_i18n().zh)),
                                    "sugar": self.ingredients.iter().find(|i| i.name == CookingIngredientName::Sugar).and_then(|object| Some(object.to_approx_unit_i18n().zh)),
                                    "soy_sauce": self.ingredients.iter().find(|i| i.name == CookingIngredientName::SoySauce).and_then(|object| Some(object.to_approx_unit_i18n().zh)),
                                    "sesame": self.ingredients.iter().find(|i| i.name == CookingIngredientName::Sesame).and_then(|object| Some(object.to_approx_unit_i18n().zh)),
                                    "sesame_oil": self.ingredients.iter().find(|i| i.name == CookingIngredientName::SesameOil).and_then(|object| Some(object.to_approx_unit_i18n().zh)),
                                    "carrot": self.ingredients.iter().find(|i| i.name == CookingIngredientName::Carrot).and_then(|object| Some(object.to_approx_unit_i18n().zh)),
                                })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                                tts_script.ko = reg.render_template(&self.tts_script.ko, &json!({
                                    "salt": self.ingredients.iter().find(|i| i.name == CookingIngredientName::Salt).and_then(|object| Some(object.to_approx_unit_i18n().ko)),
                                    "pepper": self.ingredients.iter().find(|i| i.name == CookingIngredientName::Pepper).and_then(|object| Some(object.to_approx_unit_i18n().ko)),
                                    "sugar": self.ingredients.iter().find(|i| i.name == CookingIngredientName::Sugar).and_then(|object| Some(object.to_approx_unit_i18n().ko)),
                                    "soy_sauce": self.ingredients.iter().find(|i| i.name == CookingIngredientName::SoySauce).and_then(|object| Some(object.to_approx_unit_i18n().ko)),
                                    "sesame": self.ingredients.iter().find(|i| i.name == CookingIngredientName::Sesame).and_then(|object| Some(object.to_approx_unit_i18n().ko)),
                                    "sesame_oil": self.ingredients.iter().find(|i| i.name == CookingIngredientName::SesameOil).and_then(|object| Some(object.to_approx_unit_i18n().ko)),
                                    "carrot": self.ingredients.iter().find(|i| i.name == CookingIngredientName::Carrot).and_then(|object| Some(object.to_approx_unit_i18n().ko)),
                                })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                            }
                            Some(rev) => {
                                let mut ingredients_updated: Vec<(&str, &str)> = vec![];
                                for entity in &rev.entities {
                                    match entity {
                                        CookingRevisionEntity::Add(ing) => {
                                            if link.main.name == ing.name {
                                                let amount_diff = link.main.unit.add(ing.unit).unwrap();
                                            }
                                            // let orig = self.ingredients.iter().find(|i| i.name == ing.name).unwrap();
                                        }
                                        CookingRevisionEntity::Remove(ing) => {}
                                        _ => {}
                                    }
                                }
                                if ingredients_updated.len() == 0 {
                                    return Ok(SmartSpeakerTaskResult::with_tts(
                                        SmartSpeakerTaskResultCode::StepFailed,
                                        tts_script,
                                    ));
                                }
                                tts_script.en = reg.render_template(&self.tts_script.en, &json!(ingredients_updated)).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                            }
                        }
                    }
                    CookingActionDetail::ExplainMutableTime(criteria) => {
                        match &self.current_revision {
                            None => {
                                tts_script.en = reg.render_template(&self.tts_script.en, &json!({
                                    "time": criteria.time
                                })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                                tts_script.ja = reg.render_template(&self.tts_script.ja, &json!({
                                    "time": criteria.time
                                })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                                tts_script.zh = reg.render_template(&self.tts_script.zh, &json!({
                                    "time": criteria.time
                                })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                                tts_script.ko = reg.render_template(&self.tts_script.ko, &json!({
                                    "time": criteria.time
                                })).map_err(|e| anyhow!("failed to render template: {}", e)).unwrap();
                            }
                            Some(rev) => {
                                for entity in &rev.entities {
                                    match entity {
                                        CookingRevisionEntity::Add(ing) => {
                                            if ing.name == criteria.name {
                                                let origin = self.ingredients.iter().find(|i| i.name == criteria.name).unwrap();
                                                // let amount_diff
                                                // criteria.time
                                            }
                                        }
                                        CookingRevisionEntity::Remove(ing) => {
                                            if ing.name == criteria.name {

                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
                Ok(SmartSpeakerTaskResult::with_tts(
                    SmartSpeakerTaskResultCode::StepSuccess,
                    tts_script,
                ))
            },
            _ => {
                Ok(SmartSpeakerTaskResult::with_tts(
                    SmartSpeakerTaskResultCode::StepFailed,
                    tts_script,
                ))
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
        for content in contents {
            match self.detail {
                CookingActionDetail::MeasureWholeIngredient => {
                    match content.object_type {
                        DetectableObject::Carrot => {
                            self.ingredients.iter().find(|i| i.name == CookingIngredientName::Carrot).unwrap();

                        }
                        _ => {}
                    }
                }
                CookingActionDetail::MeasureCutIngredient => {

                }
                _ => {}
            }

        }
        Ok(SmartSpeakerTaskResult::with_tts_and_revision(
            SmartSpeakerTaskResultCode::StepSuccess,
            self.tts_script.clone(),
            Box::new(CookingRevision::new(revisions)),
        ))
    }
}

impl ActionExecutable for VisionBasedIngredientMeasureAction {
    fn execute(&self) -> Result<SmartSpeakerTaskResult> {
        if self.has_cancelled() {
            return Ok(SmartSpeakerTaskResult::new(
                SmartSpeakerTaskResultCode::Cancelled));
        }
        if self.has_request_repeat() {
            return Ok(SmartSpeakerTaskResult::new(
                SmartSpeakerTaskResultCode::RepeatPrevious));
        }
        return match &self.current_content {
            None => {
                Ok(SmartSpeakerTaskResult::with_tts(
                    SmartSpeakerTaskResultCode::StepFailed,
                    self.tts_script.clone(),
                ))
            }
            Some(content) => {
                match &content.action {
                    VisionAction::None => {
                        Ok(SmartSpeakerTaskResult::with_tts(
                            SmartSpeakerTaskResultCode::StepSuccess,
                            self.tts_script.clone(),
                        ))
                    }
                    VisionAction::ObjectDetectionWithAruco(detectable) => {
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

    pub(crate) fn build(&self, menu: IntentCookingMenu) -> Vec<Box<dyn ActionExecutable>> {
        let mut steps: Vec<Box<dyn ActionExecutable>> = vec![];
        steps.push(
            Box::new(ExplainRecipeAction::new(
                vec![],
                CookingActionDetail::None,
                SmartSpeakerI18nText::new()
                    .en(&format!("Let's start cooking {}. Tell me when you ready with an answer such as 'ok'.", menu.to_i18n().en))
                    .ja(&format!("{}の調理を始めます。準備ができたら「オッケー」などの答えで教えてください。", menu.to_i18n().ja))
                    .zh(&format!("让我们开始做{}。准备好了就告诉我，比如说“好的”。", menu.to_i18n().zh))
                    .ko(&format!("{} 요리를 시작합니다. 준비가 되면 '오케이'와 같은 대답으로 알려주세요.", menu.to_i18n().ko))
            )));
        steps.push(
            Box::new(ExplainRecipeAction::new(
                menu.to_ingredient(),
                CookingActionDetail::ExplainNonMutableIngredient,
                SmartSpeakerI18nText::new()
                    .ko("요리 재료 설명을 시작합니다. {{additional_explain}} 가 필요합니다. 다음으로 넘어갈 준비가 되었으면 알려주세요. 다시 한 번 들으시려면 '다시 알려 줘' 라고 말씀해주세요.")
                    .en("Let's start explaining ingredients. {{additional_explain}} is required. Let me know when you are ready to proceed. If you want to hear it again, please say 'tell me again'.")
                    .ja("食材の説明を始めます。{{additional_explain}} が必要です。次に進む準備ができたら教えてください。もう一度聞きたい場合は、「もう一度教えて」と言ってください。")
                    .zh("让我们开始解释食材。{{additional_explain}} 是必需的。准备好后请告诉我。如果你想再听一遍，请说“再告诉我一遍”。")
            )));
        steps.push(
            Box::new(ExplainRecipeAction::new(
                vec![],
                CookingActionDetail::None,
                SmartSpeakerI18nText::new()
                    .ko("먼저 당근을 준비합니다.")
                    .en("First, prepare the carrots.")
                    .ja("まず人参を用意します。")
                    .zh("首先准备胡萝卜。")
            )));
        if self.vision {
            steps.push(
                Box::new(ExplainRecipeAction::new(
                    vec![],
                    CookingActionDetail::None,
                    SmartSpeakerI18nText::new()
                        .ko("보다 정확한 레시피 안내를 위해 요리 재료의 크기 측정을 시작합니다. 당근을 측정용 도마 위에 올려주세요. 준비가 되면 '오케이'와 같은 대답으로 알려주세요.")
                        .en("To provide more accurate recipe guidance, we will start measuring the size of the cooking ingredients. Place the carrots on the measuring chopping board. Let us know when it's ready with a response like 'okay'")
                        .ja("より正確なレシピ案内のために食材の大きさを測定し始めます。人参を測定用のまな板の上に置いてください。準備ができたら「オッケー」などの答えで教えてください。")
                        .zh("为了提供更准确的食谱指导，我们将开始测量烹饪食材的大小。把胡萝卜放在量板上。准备好了就告诉我，比如说“好的”。")
                ))
            );
            steps.push(
                Box::new(ExplainRecipeAction::new(
                    vec![],
                    CookingActionDetail::None,
                    SmartSpeakerI18nText::new()
                        .ko("측정중입니다. 움직이지 말고 기다려 주세요.")
                        .en("I will start measuring. Please do not move and wait.")
                        .ja("測定を始めます。動かずにお待ちください。")
                        .zh("我将开始测量。请不要动，等一下。")
                ))
            );
            steps.push(
                Box::new(VisionBasedIngredientMeasureAction::new(
                    vec![menu.to_ingredient().iter().find(|i| i.name == CookingIngredientName::Carrot).unwrap().clone()],
                    CookingActionDetail::MeasureWholeIngredient,
                    VisionAction::ObjectDetectionWithAruco(DetectableObject::Carrot),
                    SmartSpeakerI18nText::new()
                        .ko("확인했습니다.")
                        .en("Checked.")
                        .ja("確認しました。")
                        .zh("确认了。")
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
                        .ko("보다 정확한 레시피 안내를 위해 요리 재료의 크기 측정을 시작합니다. 잘라낸 한 조각을 측정용 도마 위에 올려주세요. 준비가 되면 '오케이'와 같은 대답으로 알려주세요.")
                        .en("To provide more accurate recipe guidance, we will start measuring the size of the cooking ingredients. Place one of the cut pieces on the measuring chopping board. Let us know when it's ready with a response like 'okay'")
                        .ja("より正確なレシピ案内のために食材の大きさを測定し始めます。切り分けた一つを測定用のまな板の上に置いてください。準備ができたら「オッケー」などの答えで教えてください。")
                        .zh("为了提供更准确的食谱指导，我们将开始测量烹饪食材的大小。把切好的一块放在量板上。准备好了就告诉我，比如说“好的”。")
                ))
            );
            steps.push(
                Box::new(ExplainRecipeAction::new(
                    vec![],
                    CookingActionDetail::None,
                    SmartSpeakerI18nText::new()
                        .en("I will start measuring. Please do not move and wait.")
                        .ja("測定を始めます。動かずにお待ちください。")
                        .zh("我将开始测量。请不要动，等一下。")
                        .ko("측정중입니다. 움직이지 말고 기다려 주세요.")
                ))
            );
            steps.push(
                Box::new(VisionBasedIngredientMeasureAction::new(
                    vec![CookingIngredient::new(
                        CookingIngredientName::Carrot,
                        CookingIngredientAmount::MilliGram(10))],
                    CookingActionDetail::MeasureCutIngredient,
                    VisionAction::ObjectDetectionWithAruco(DetectableObject::Carrot),
                    SmartSpeakerI18nText::new()
                        .ko("확인했습니다.")
                        .en("Checked.")
                        .ja("確認しました。")
                        .zh("确认了。")
                ))
            );
        }
        steps.push(
            Box::new(ExplainRecipeAction::new(
                vec![menu.to_ingredient().iter().find(|i| i.name == CookingIngredientName::Carrot).unwrap().clone()],
                CookingActionDetail::ExplainMutableTime(CookingIngredientTime::new(
                    CookingIngredientName::Carrot,
                    10)),
                SmartSpeakerI18nText::new() // replace to template!!
                    .ko("손질한 당근을 끓는 물에 약 {{time}}분간 삶아주세요.")
                    .en("Boil the carrots in boiling water for about {{time}} minutes.")
                    .ja("人参を沸いた水に約{{time}}分間茹でます。")
                    .zh("把胡萝卜放在沸水里煮约{{time}}分钟。")
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
                    .ko("삶은 당근을 보울에 담아 소금 {{salt}}, 후추 {{pepper}}, 참기름 {{sesame_oil}}을 넣고 섞어주세요.")
                    .en("Put the boiled carrots in a bowl and add {{salt}} of salt, {{pepper}} of pepper, and {{sesame_oil}} of sesame oil.")
                    .ja("茹でた人参をボウルに入れて塩{{salt}}、コショウ{{pepper}}、ごま油{{sesame_oil}}を入れて混ぜます。")
                    .zh("把煮好的胡萝卜放在碗里，加{{salt}}的盐，{{pepper}}的胡椒粉，{{sesame_oil}}的芝麻油。")
            )));
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
