use std::fmt::{Debug};
use anyhow::{anyhow, Result};
use crate::smart_speaker::models::core_model::WaitingInteraction;
use crate::smart_speaker::models::intent_model::IntentCookingMenu;
use crate::smart_speaker::models::step_model::generic_step::{ActionExecutable, ActionTriggerType};
use crate::smart_speaker::models::task_model::{SmartSpeakerTaskResult, SmartSpeakerTaskResultCode};
use crate::smart_speaker::models::vision_model::{DetectableObject, VisionAction, VisionObject};
use crate::smart_speaker::models::message_model::*;
use crate::smart_speaker::models::revision_model::cooking_revision::CookingRevision;
use crate::smart_speaker::models::revision_model::Revision;
use crate::smart_speaker::models::task_model::cooking_task::CookingIngredient;

#[derive(Debug, Clone)]
pub(crate) struct ExplainRecipeAction {
    pub(crate) ingredients: Vec<CookingIngredient>,
    pub(crate) tts_script: SmartSpeakerI18nText,
    pub(crate) current_content: Option<IntentContent>,
    pub(crate) current_revision: Option<CookingRevision>,
    cancelled: bool,
}

impl ExplainRecipeAction {
    pub(crate) fn new(ingredients: Vec<CookingIngredient>, text: SmartSpeakerI18nText) -> Self {
        ExplainRecipeAction {
            ingredients,
            tts_script: text,
            current_content: None,
            current_revision: None,
            cancelled: false,
        }
    }
}

impl ActionExecutable for ExplainRecipeAction {
    fn execute(&self) -> Result<SmartSpeakerTaskResult> {
        if self.has_cancelled() {
            return Ok(SmartSpeakerTaskResult::new(
                SmartSpeakerTaskResultCode::Cancelled));
        }
        match &self.current_content {
            Some(intent) => {
                Ok(SmartSpeakerTaskResult::with_tts(
                    SmartSpeakerTaskResultCode::StepSuccess,
                    self.tts_script.clone(),
                ))
            },
            _ => {
                Ok(SmartSpeakerTaskResult::with_tts(
                    SmartSpeakerTaskResultCode::StepFailed,
                    self.tts_script.clone(),
                ))
            }
        }
    }

    fn feed(&mut self, content: Box<dyn Content>, revision: Option<Box<dyn Revision>>) -> Result<()> {
        self.check_cancelled(&content)?;
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

    fn expose_tts_script(&self) -> Result<SmartSpeakerI18nText> {
        Ok(self.tts_script.clone())
    }

    fn try_expose_vision_actions(&self) -> Result<Vec<VisionAction>> {
        Err(anyhow!("Not a vision action"))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct VisionBasedIngredientMeasureAction {
    pub(crate) tts_script: SmartSpeakerI18nText,
    pub(crate) vision_action: VisionAction,
    pub(crate) current_content: Option<VisionContent>,
    pub(crate) current_revision: Option<CookingRevision>,
    cancelled: bool,
}

impl VisionBasedIngredientMeasureAction {
    pub(crate) fn new(vision_action: VisionAction, text: SmartSpeakerI18nText) -> Self {
        VisionBasedIngredientMeasureAction {
            tts_script: text,
            vision_action,
            current_content: None,
            current_revision: None,
            cancelled: false,
        }
    }

    pub(crate) fn handle_vision_contents(&self, contents: &Vec<VisionObject>) -> Result<SmartSpeakerTaskResult> {
        // write logic here
        Ok(SmartSpeakerTaskResult::with_tts(
            SmartSpeakerTaskResultCode::StepSuccess,
            self.tts_script.clone(),
        ))
    }
}

impl ActionExecutable for VisionBasedIngredientMeasureAction {
    fn execute(&self) -> Result<SmartSpeakerTaskResult> {
        if self.has_cancelled() {
            return Ok(SmartSpeakerTaskResult::new(
                SmartSpeakerTaskResultCode::Cancelled));
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
                SmartSpeakerI18nText::new()
                    .en(&format!("Let's start cooking {}. Tell me when you ready with an answer such as 'ok'.", menu.to_i18n().en))
                    .ja(&format!("{}の調理を始めます。準備ができたら「オッケー」などの答えで教えてください。", menu.to_i18n().ja))
                    .zh(&format!("让我们开始做{}。准备好了就告诉我，比如说“好的”。", menu.to_i18n().zh))
                    .ko(&format!("{} 요리를 시작합니다. 준비가 되면 '오케이'와 같은 대답으로 알려주세요.", menu.to_i18n().ko))
            )));
        steps.push(
            Box::new(ExplainRecipeAction::new(
                menu.to_ingredient(),
                SmartSpeakerI18nText::new()
                    .ko("요리 재료 설명을 시작합니다.")
                    .en("Let's start explaining ingredients.")
                    .ja("食材の説明を始めます。")
                    .zh("让我们开始解释食材。")
            )));
        if self.vision {
            steps.push(
                Box::new(ExplainRecipeAction::new(
                    vec![],
                    SmartSpeakerI18nText::new()
                        .en("Start measuring the size of the ingredients. Please put the carrot on the measuring board. When you are ready, please let me know with an answer such as 'ok'.")
                        .ja("食材の大きさを測定し始めます。人参を測定用のまな板の上に置いてください。準備ができたら「オッケー」などの答えで教えてください。")
                        .zh("让我们开始测量食材的大小。请把胡萝卜放在量板上。准备好了就告诉我，比如说“好的”。")
                        .ko("요리 재료의 크기 측정을 시작합니다. 당근을 측정용 도마 위에 올려주세요. 준비가 되면 '오케이'와 같은 대답으로 알려주세요.")
                ))
            );
            steps.push(
                Box::new(ExplainRecipeAction::new(
                    vec![],
                    SmartSpeakerI18nText::new()
                        .en("I will start measuring. Please do not move and wait.")
                        .ja("測定を始めます。動かずにお待ちください。")
                        .zh("我将开始测量。请不要动，等一下。")
                        .ko("측정중입니다. 움직이지 말고 기다려 주세요.")
                ))
            );
            steps.push(
                Box::new(VisionBasedIngredientMeasureAction::new(
                    VisionAction::ObjectDetectionWithAruco(DetectableObject::Carrot),
                    SmartSpeakerI18nText::new()
                        .en("")
                        .ja("")
                        .zh("")
                        .ko("확인했습니다.")
                ))
            );
        }
        steps.push(
            Box::new(ExplainRecipeAction::new(
                vec![],
                SmartSpeakerI18nText::new()
                    .ko("계속해서 당근을 먹기 좋은 크기로 썰어주세요.")
                    .en("Please continue to cut the carrots into bite-sized pieces.")
                    .ja("人参を食べやすい大きさに切り続けてください。")
                    .zh("请继续把胡萝卜切成一口大小的块。")
            )));
        if self.vision {
            steps.push(
                Box::new(ExplainRecipeAction::new(
                    vec![],
                    SmartSpeakerI18nText::new()
                        .en("Start measuring the size of the ingredients. Place a cut piece on the measuring chopping board. Let us know when it's ready with a response like 'okay'")
                        .ja("食材の大きさを測定し始めます。切り分けた一つを測定用のまな板の上に置いてください。準備ができたら「オッケー」などの答えで教えてください。")
                        .zh("让我们开始测量食材的大小。把切好的一块放在量板上。准备好了就告诉我，比如说“好的”。")
                        .ko("요리 재료의 크기 측정을 시작합니다. 잘라낸 한 조각을 측정용 도마 위에 올려주세요. 준비가 되면 '오케이'와 같은 대답으로 알려주세요.")
                ))
            );
            steps.push(
                Box::new(ExplainRecipeAction::new(
                    vec![],
                    SmartSpeakerI18nText::new()
                        .en("I will start measuring. Please do not move and wait.")
                        .ja("測定を始めます。動かずにお待ちください。")
                        .zh("我将开始测量。请不要动，等一下。")
                        .ko("측정중입니다. 움직이지 말고 기다려 주세요.")
                ))
            );
            steps.push(
                Box::new(VisionBasedIngredientMeasureAction::new(
                    VisionAction::ObjectDetectionWithAruco(DetectableObject::Carrot),
                    SmartSpeakerI18nText::new()
                        .en("")
                        .ja("")
                        .zh("")
                        .ko("확인했습니다.")
                ))
            );
        }
        steps.push(
            Box::new(ExplainRecipeAction::new(
                vec![],
                SmartSpeakerI18nText::new() // replace to template!!
                    .ko("손질한 당근을 끓는 물에 약 5분간 삶아주세요.")
                    .en("Boil the carrots in boiling water for about 5 minutes.")
                    .ja("人参を沸いた水に約5分間茹でます。")
                    .zh("把胡萝卜放在沸水里煮约5分钟。")
            )));

        steps
    }
}
