use std::str::FromStr;
use anyhow::{anyhow, Error, Result};
use crate::smart_speaker::models::core_model::PendingType;
use crate::smart_speaker::models::intent_model::{IntentAction, IntentCookingMenu, IntentSlot};
use crate::utils::message_util::{Content, ContentType, IntentContent};

// #[derive(Debug, PartialEq)]
// pub(crate) enum Context {
//     Task(SmartSpeakerTaskContext),
// }
//
// #[derive(Debug, PartialEq)]
// pub(crate) enum SmartSpeakerTaskContext {
//     Viewing(ViewingTaskContext), // for test
//     Cooking(CookingTaskContext),
//     Unknown,
// }
//
// impl FromStr for SmartSpeakerTaskContext {
//     type Err = Error;
//
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         match s.to_lowercase().as_str() {
//             "viewing" => Ok(SmartSpeakerTaskContext::Viewing(ViewingTaskContext {
//                 agent: None,
//                 target: None,
//             })),
//             "cooking" => Ok(SmartSpeakerTaskContext::Cooking(CookingTaskContext {})),
//             _ => Err(anyhow!("invalid task context")),
//         }
//     }
// }

#[derive(Debug, PartialEq)]
pub(crate) enum SmartSpeakerTaskResultCodes {
    Exit(String),
    Wait(PendingType),
}

#[derive(Debug, PartialEq)]
pub(crate) struct ViewingTaskContext {
    pub(crate) agent: Option<String>,
    pub(crate) target: Option<String>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct CookingTaskContext {
    pub(crate) menu: IntentCookingMenu,
}

pub(crate) trait Task: Send {
    fn init(&mut self) -> Result<SmartSpeakerTaskResultCodes>;
    fn next(&mut self, content: Option<Box<dyn Content>>) -> Result<SmartSpeakerTaskResultCodes>;
    fn failed(&mut self, content: Option<Box<dyn Content>>) -> Result<SmartSpeakerTaskResultCodes>;
    fn exit(&self) -> Result<SmartSpeakerTaskResultCodes>;
}

pub(crate) struct ViewingTask {
    pub(crate) context: ViewingTaskContext,
}

impl ViewingTask {
    pub(crate) fn new(content: IntentContent) -> Result<Self> {
        todo!()
    }
}

impl Task for ViewingTask {
    fn init(&mut self) -> Result<SmartSpeakerTaskResultCodes> {
        Ok(SmartSpeakerTaskResultCodes::Wait(PendingType::Speak))
    }

    fn next(&mut self, content: Option<Box<dyn Content>>) -> Result<SmartSpeakerTaskResultCodes> {
        Ok(SmartSpeakerTaskResultCodes::Exit("viewing task next".to_string()))
    }

    fn failed(&mut self, content: Option<Box<dyn Content>>) -> Result<SmartSpeakerTaskResultCodes> {
        Ok(SmartSpeakerTaskResultCodes::Exit("viewing task failed".to_string()))
    }

    fn exit(&self) -> Result<SmartSpeakerTaskResultCodes> {
        Ok(SmartSpeakerTaskResultCodes::Exit("viewing task exit".to_string()))
    }
}

pub(crate) struct CookingTask {
    pub(crate) context: CookingTaskContext,
    pub(crate) waiting_content: PendingType
}

impl CookingTask {
    pub(crate) fn new(content: IntentContent) -> Result<Self> {
        match content.entities.get(0) {
            None => { Err(anyhow!("failed")) }
            Some(entity) => {
                Ok(CookingTask {
                    context: CookingTaskContext {
                        menu: entity.as_any().downcast_ref::<IntentCookingMenu>().unwrap().clone(),
                    },
                    waiting_content: PendingType::Speak,
                })
            }
        }
    }

    fn update(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Task for CookingTask {
    fn init(&mut self) -> Result<SmartSpeakerTaskResultCodes> {
        dbg!("cooking task init");
        Ok(SmartSpeakerTaskResultCodes::Wait(PendingType::Speak))
    }

    fn next(&mut self, content: Option<Box<dyn Content>>) -> Result<SmartSpeakerTaskResultCodes> {
        match content {
            None => {
                Ok(SmartSpeakerTaskResultCodes::Wait(PendingType::Speak))
            }
            Some(c) => {
                match c.as_any().downcast_ref::<IntentContent>() {
                    None => {
                        Ok(SmartSpeakerTaskResultCodes::Wait(PendingType::Speak))
                    }
                    Some(intent) => {
                        match intent.intent {
                            IntentAction::Cancel => {
                                self.exit()
                            }
                            _ => {
                                Ok(SmartSpeakerTaskResultCodes::Wait(PendingType::Speak))
                            }
                        }
                    }
                }
            }
        }
    }

    fn failed(&mut self, content: Option<Box<dyn Content>>) -> Result<SmartSpeakerTaskResultCodes> {
        Ok(SmartSpeakerTaskResultCodes::Wait(self.waiting_content.clone()))
    }

    fn exit(&self) -> Result<SmartSpeakerTaskResultCodes> {
        Ok(SmartSpeakerTaskResultCodes::Exit("cooking task exit".to_string()))
    }
}
