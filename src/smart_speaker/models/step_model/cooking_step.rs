use std::fmt::{Debug};
use anyhow::{anyhow, Result};
use handlebars::Handlebars;
use crate::smart_speaker::models::intent_model::IntentCookingMenu;
use crate::smart_speaker::models::step_model::generic_step::{ActionExecutable, ActionTriggerType};
use crate::smart_speaker::models::task_model::{SmartSpeakerTaskResult, SmartSpeakerTaskResultCode};
use crate::smart_speaker::models::vision_model::{DetectableObject, VisionAction, VisionObject};
use crate::smart_speaker::models::message_model::*;
use crate::smart_speaker::models::revision_model::cooking_revision::CookingRevision;
use crate::smart_speaker::models::revision_model::Revision;
use crate::smart_speaker::models::task_model::cooking_task::CookingIngredient;

pub(crate) enum CookingActionDetail {
    None,
    ExplainNonMutableIngredient,
    ExplainMutableIngredient,
    ExplainMutableTime,
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
    pub(crate) detail: CookingActionDetail,
    pub(crate) vision_action: VisionAction,
    pub(crate) tts_script: SmartSpeakerI18nText,
    pub(crate) current_content: Option<VisionContent>,
    pub(crate) current_revision: Option<CookingRevision>,
    cancelled: bool,
}

impl VisionBasedIngredientMeasureAction {
    pub(crate) fn new(detail: CookingActionDetail, vision_action: VisionAction, text: SmartSpeakerI18nText) -> Self {
        VisionBasedIngredientMeasureAction {
            detail,
            vision_action,
            tts_script: text,
            current_content: None,
            current_revision: None,
            cancelled: false,
        }
    }

    pub(crate) fn handle_vision_contents(&self, contents: &Vec<VisionObject>) -> Result<SmartSpeakerTaskResult> {
        // write logic here
        match self.detail {
            CookingActionDetail::MeasureWholeIngredient => {

            }
            CookingActionDetail::MeasureCutIngredient => {

            }
            _ => {}
        }
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
                    .ko("요리 재료 설명을 시작합니다. {{additional_explain}}")
                    .en("Let's start explaining ingredients. {{additional_explain}}")
                    .ja("食材の説明を始めます。{{additional_explain}}")
                    .zh("让我们开始解释食材。{{additional_explain}}")
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
                    .ja("人参を食べやすい大きさに切り続けてください。")
                    .zh("请继续把胡萝卜切成一口大小的块。")
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
                vec![],
                CookingActionDetail::ExplainMutableTime,
                SmartSpeakerI18nText::new() // replace to template!!
                    .ko("손질한 당근을 끓는 물에 약 {{time}}분간 삶아주세요.")
                    .en("Boil the carrots in boiling water for about {{time}} minutes.")
                    .ja("人参を沸いた水に約{{time}}分間茹でます。")
                    .zh("把胡萝卜放在沸水里煮约{{time}}分钟。")
            )));
        steps.push(
            Box::new(ExplainRecipeAction::new(
                vec![],
                CookingActionDetail::ExplainMutableIngredient,
                SmartSpeakerI18nText::new() // replace to template!!
                    .ko("삶은 당근을 보울에 담아 소금 {{salt}} 스푼, 후추 {{pepper}} 스푼, 참기름 {{sesame_oil}} 스푼을 넣고 섞어주세요.")
                    .en("Put the boiled carrots in a bowl and add {{salt}} spoons of salt, {{pepper}} spoons of pepper, and {{sesame_oil}} spoons of sesame oil.")
                    .ja("茹でた人参をボウルに入れて塩{{salt}}スプーン、コショウ{{pepper}}スプーン、ごま油{{sesame_oil}}スプーンを入れて混ぜます。")
                    .zh("把煮好的胡萝卜放在碗里，加{{salt}}勺盐，{{pepper}}勺胡椒粉，{{sesame_oil}}勺芝麻油。")
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
