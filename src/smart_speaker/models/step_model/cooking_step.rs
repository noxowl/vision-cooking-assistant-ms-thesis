#[derive(Debug, Clone)]
pub(crate) struct CookingStep {
    pub(crate) waiting_for: PendingType,
    pub(crate) action: CookingAction,
}

impl CookingStep {
    pub(crate) fn new(action: CookingAction) -> Self {
        CookingStep {
            waiting_for: PendingType::Speak,
            action
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum CookingAction {
    None,
    Explain(ExplainStepExecutable),
    WaitForConfirm,
    WaitForVision(Box<dyn StepExecutable>),
}


pub(crate) trait StepExecutable: Send {
    fn execute(&self) -> Result<SmartSpeakerTaskResultCodes>;
    fn feed(&mut self, content: Box<dyn Content>) -> Result<()>;
    fn clone_box(&self) -> Box<dyn StepExecutable>;
    fn as_any(&self) -> &dyn std::any::Any;
}

impl Debug for dyn StepExecutable {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "StepExecutables")
    }
}

impl PartialEq for dyn StepExecutable {
    fn eq(&self, other: &Self) -> bool {
        true
    }
}

impl Clone for Box<dyn StepExecutable> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ExplainStepExecutable {
    pub(crate) text: String,
    pub(crate) current_content: Option<IntentContent>,
}

impl ExplainStepExecutable {
    pub(crate) fn new(text: String) -> Self {
        ExplainStepExecutable {
            text,
            current_content: None,
        }
    }
}

impl StepExecutable for ExplainStepExecutable {
    fn clone_box(&self) -> Box<dyn StepExecutable> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn execute(&self) -> Result<SmartSpeakerTaskResultCodes> {
        Ok(SmartSpeakerTaskResultCodes::Exit(self.text.clone()))
    }

    fn feed(&mut self, content: Box<dyn Content>) -> Result<()> {
        match content.as_any().downcast_ref::<IntentContent>() {
            None => {}
            Some(intent) => {
                self.current_content = Some(intent.clone());
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct WaitForVisionStepExecutables {
    pub(crate) vision_action: VisionAction,
    pub(crate) current_content: Option<VisionContent>,
}

impl WaitForVisionStepExecutables {
    pub(crate) fn new(vision_action: VisionAction) -> Self {
        WaitForVisionStepExecutables {
            vision_action,
            current_content: None,
        }
    }
}

impl StepExecutable for WaitForVisionStepExecutables {
    fn execute(&self) -> Result<SmartSpeakerTaskResultCodes> {
        return match &self.current_content {
            None => {
                Ok(SmartSpeakerTaskResultCodes::Wait(PendingType::Vision(vec![])))
            }
            Some(content) => {
                match &content.action {
                    VisionAction::None => {
                        Ok(SmartSpeakerTaskResultCodes::Wait(PendingType::Vision(vec![])))
                    }
                    VisionAction::ObjectDetectionWithAruco(detectable) => {
                        for content in &content.entities {
                            match content.as_any().downcast_ref::<VisionObject>() {
                                None => {
                                }
                                Some(vision_object) => {
                                }
                            }
                        }
                        Ok(SmartSpeakerTaskResultCodes::Wait(PendingType::Vision(vec![])))
                    }
                }
            }
        }
    }

    fn feed(&mut self, content: Box<dyn Content>) -> Result<()> {
        match content.as_any().downcast_ref::<VisionContent>() {
            None => {}
            Some(vision) => {
                self.current_content = Some(vision.clone());
            }
        }
        Ok(())
    }

    fn clone_box(&self) -> Box<dyn StepExecutable> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}