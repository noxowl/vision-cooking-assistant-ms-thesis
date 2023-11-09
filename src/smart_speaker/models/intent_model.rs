use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use crate::smart_speaker::models::message_model::SmartSpeakerI18nText;

pub(crate) trait IntentSlot: Send {
    fn clone_box(&self) -> Box<dyn IntentSlot>;
    fn as_any(&self) -> &dyn std::any::Any;
}

impl Debug for dyn IntentSlot {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "IntentSlot")
    }
}

impl PartialEq for dyn IntentSlot {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Clone for Box<dyn IntentSlot> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum IntentPlace {
    Room,
}

impl IntentSlot for IntentPlace {
    fn clone_box(&self) -> Box<dyn IntentSlot> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl FromStr for IntentPlace {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "room" => Ok(IntentPlace::Room),
            "へや" => Ok(IntentPlace::Room),
            "部屋" => Ok(IntentPlace::Room),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum IntentCookingMenu {
    CarrotSalad,
}

impl IntentCookingMenu {
    pub(crate) fn to_i18n(&self) -> SmartSpeakerI18nText {
        match self {
            IntentCookingMenu::CarrotSalad => SmartSpeakerI18nText::new()
                .en("Carrot salad")
                .ja("にんじんサラダ")
                .zh("胡萝卜沙拉")
                .ko("당근 샐러드")
        }
    }
}

impl IntentSlot for IntentCookingMenu {
    fn clone_box(&self) -> Box<dyn IntentSlot> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl FromStr for IntentCookingMenu {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "carrot salad" => Ok(IntentCookingMenu::CarrotSalad),
            "にんじんサラダ" => Ok(IntentCookingMenu::CarrotSalad),
            "人参サラダ" => Ok(IntentCookingMenu::CarrotSalad),
            "にんじんのサラダ" => Ok(IntentCookingMenu::CarrotSalad),
            "にんじんりようり" => Ok(IntentCookingMenu::CarrotSalad),
            _ => Err(()),
        }
    }
}

impl Display for IntentCookingMenu {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            IntentCookingMenu::CarrotSalad => write!(f, "にんじんサラダ"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum IntentAction {
    None,
    TurnOn,
    TurnOff,
    Purchase,
    Cancel,
    WhatYouSee,
    CookingTask,
    Confirm,
    Next,
    Previous,
}

impl FromStr for IntentAction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "turn on" => Ok(IntentAction::TurnOn),
            "turn off" => Ok(IntentAction::TurnOff),
            "purchase" => Ok(IntentAction::Purchase),
            "buy" => Ok(IntentAction::Purchase),
            "つけて" => Ok(IntentAction::TurnOn),
            "けして" => Ok(IntentAction::TurnOff),
            "つけてください" => Ok(IntentAction::TurnOn),
            "けしてください" => Ok(IntentAction::TurnOff),
            "つけてくれ" => Ok(IntentAction::TurnOn),
            "けしてくれ" => Ok(IntentAction::TurnOff),
            "見えているもの" => Ok(IntentAction::WhatYouSee),
            "料理作業" => Ok(IntentAction::CookingTask),
            "承認" => Ok(IntentAction::Confirm),
            "取り消し" => Ok(IntentAction::Cancel),
            "次" => Ok(IntentAction::Next),
            "繰り返し" => Ok(IntentAction::Previous),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq)]
enum IntentObject {
    Light,
}

impl FromStr for IntentObject {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "light" => Ok(IntentObject::Light),
            "ライト" => Ok(IntentObject::Light),
            "照明" => Ok(IntentObject::Light),
            _ => Err(()),
        }
    }
}
