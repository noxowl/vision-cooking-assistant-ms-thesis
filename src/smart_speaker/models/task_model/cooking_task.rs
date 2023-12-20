use std::ops::Div;
use anyhow::{anyhow, Result};
use crate::smart_speaker::models::core_model::WaitingInteraction;
use crate::smart_speaker::models::intent_model::{IntentAction, IntentCookingMenu};
use crate::smart_speaker::models::step_model::generic_step::{ActionExecutable, ActionTriggerType};
use crate::smart_speaker::models::task_model::{SmartSpeakerTaskResult, SmartSpeakerTaskResultCode, SmartSpeakerTaskType, Task};
use crate::smart_speaker::models::message_model::*;
use crate::smart_speaker::models::revision_model::cooking_revision::{CookingRevisionEntity, CookingRevisionEntityProperty};
use crate::smart_speaker::models::revision_model::Revision;
use crate::smart_speaker::models::speak_model::MachineSpeechBoilerplate;
use crate::smart_speaker::models::step_model::cooking_step::CookingStepBuilder;


#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SmartSpeakerMaterialProperty {
    Solid,
    Liquid,
    Powder,
    Gas,
}

#[derive(Debug, Clone)]
pub(crate) struct CookingIngredientTime {
    pub(crate) base: CookingIngredient,
    pub(crate) time: u32,
}

impl CookingIngredientTime {
    pub(crate) fn new(base: CookingIngredient, time: u32) -> Self {
        CookingIngredientTime {
            base,
            time,
        }
    }

    pub(crate) fn calc_time_by_revision(&self, rev: &CookingRevisionEntity) -> Option<CookingIngredientTime> {
        // calculate the time by base time with revision as quarter. 10min is 100 u32 in time.
        // If base is mg(250)/10min and rev is sub(mg(65)), the new base time is 7.5min(75) that means the time should be reduced by 3/4.
        // The quota calculation also approximates the value so that it falls in the 4/4 range whenever possible.
        // in this situation, if base=mg(250)/10min and rev=sub(mg(65))
        // the result is mg(250)/7.5min(75)
        // if base=mg(250)/10min and rev=add(mg(65))
        // the result is mg(250)/12.5min(125)
        // if base=mg(250)/10min and rev=add(mg(250))
        // the result is mg(250)/20min(200)
        let adjustment_factor = match &rev.property {
            CookingRevisionEntityProperty::Sub(ingredient) | CookingRevisionEntityProperty::Add(ingredient) => {
                if ingredient.name != self.base.name {
                    return None;
                }
                let revised_amount = if let CookingRevisionEntityProperty::Sub(_) = &rev.property {
                    self.base.amount() - ingredient.amount()
                } else {
                    self.base.amount() + ingredient.amount()
                };
                1.0 + (revised_amount / self.base.amount()).sqrt()
            }
        };

        let new_time = ((self.time as f32).sqrt() * adjustment_factor).round() as u32;
        let bounded_time = new_time.min(300).max(30);
        Some(CookingIngredientTime::new(self.base.clone(), bounded_time))
    }

    pub(crate) fn to_human_time(&self) -> SmartSpeakerI18nText {
        // 10 is 1min and 5 is 30sec. This would make 1 = 6sec, but the seconds value we need here is to cut in increments of 10sec, so 9=54sec, but we need to return 50sec.
        // The SmartSpeakerI18nText result should be simplified to {} min {} sec when there is sec information, and {} min when there is no sec information.
        let minutes = self.time / 10;
        let remaining_seconds = (self.time % 10) * 6; // Convert to seconds, each unit is 6 seconds

        let mut rounded_seconds = 0;
        if remaining_seconds > 0 {
            rounded_seconds = 10 + ((remaining_seconds + 4) / 10) * 10; // Round to the nearest 10 seconds
        }

        if rounded_seconds > 0 {
            SmartSpeakerI18nText::new()
                .en(&format!("{} minutes {} seconds", minutes, rounded_seconds))
                .ja(&format!("{}分{}秒", minutes, rounded_seconds))
                .zh(&format!("{}分{}秒", minutes, rounded_seconds))
                .ko(&format!("{}분{}초", minutes, rounded_seconds))
        } else {
            SmartSpeakerI18nText::new()
                .en(&format!("{} minutes", minutes))
                .ja(&format!("{}分", minutes))
                .zh(&format!("{}分", minutes))
                .ko(&format!("{}분", minutes))
        }
    }
}

pub(crate) fn amount_to_approx_quarter(lhs: i32, rhs: i32) -> CookingIngredientAmountQuarter {
    // This function compares the number of LHS and RHS (the base) and returns a number in the range of 4/4.
    // The CookingIngredientAmountQuarter value is 4/4 = 1 piece.
    // For example, if the base is 1000, the range is 250, 500, 750, 1000.
    if rhs == 0 {
        return CookingIngredientAmountQuarter::new(0);
    }

    // Calculate the quarter value based on LHS divided by the base (rhs) and multiplied by 4
    let quarter_value = ((lhs as f32 / rhs as f32) * 4.0).round() as i32;

    // Ensure the quarter value is greater than or equal to 1
    let clamped_quarter = quarter_value.max(1);

    CookingIngredientAmountQuarter::new(clamped_quarter)
}

#[derive(Debug, Clone)]
pub(crate) struct CookingIngredientLinkComponent {
    pub(crate) main: CookingIngredient,
    pub(crate) components: Vec<CookingIngredient>,
}

impl CookingIngredientLinkComponent {
    pub(crate) fn new(main: CookingIngredient, components: Vec<CookingIngredient>) -> Self {
        CookingIngredientLinkComponent {
            main,
            components,
        }
    }

    pub(crate) fn calc_components_amount_by_main_revision(&self, rev: &CookingRevisionEntity) -> Vec<CookingIngredient> {
        // calculate the components(seasoning ingredients) amount by main ingredient amount with revision as quarter.
        // If main is mg(1000) and rev is sub(mg(250)), the new main amount is mg(750) that means the components connected to main should be reduced by 3/4.
        // The quota calculation also approximates the value so that it falls in the 4/4 range whenever possible.
        // in this situation, if main=1000, components=vec![salt(mg(50)), pepper(mg(50)), sesame_oil(ml(5))] and rev=sub(mg(250))
        // the result is vec![salt(mg(40)), pepper(mg(40)), sesame_oil(ml(40))]
        // if main=mg(1000), components=vec![salt(mg(5)), pepper(mg(5)), sesame_oil(ml(5))] and rev=add(mg(500))
        // the result is vec![salt(mg(70)), pepper(mg(70)), sesame_oil(ml(7))]
        // if main=mg(1000), components=vec![salt(mg(5)), pepper(mg(5)), sesame_oil(ml(5))] and rev=add(mg(1000))
        // the result is vec![salt(mg(100)), pepper(mg(100)), sesame_oil(ml(10))]
        let adjustment_factor = match &rev.property {
            CookingRevisionEntityProperty::Sub(ingredient) => {
                1.0 - ingredient.amount() / self.main.amount()
            }
            CookingRevisionEntityProperty::Add(ingredient) => {
                1.0 + ingredient.amount() / self.main.amount()
            },
        };
        dbg!(&adjustment_factor);

        // Iterate over the components and calculate the adjusted amount for each.
        let result: Vec<CookingIngredient> = self
            .components
            .iter()
            .map(|ingredient| {
                let adjusted_amount = match ingredient.unit {
                    CookingIngredientAmount::MilliGram(amount) => {
                        let amount = (amount as f32 * adjustment_factor).round() as i32;
                        CookingIngredientAmount::MilliGram(amount)
                    }
                    CookingIngredientAmount::MilliLiter(amount) => {
                        let amount = (amount as f32 * adjustment_factor).round() as i32;
                        CookingIngredientAmount::MilliLiter(amount)
                    }
                    CookingIngredientAmount::Piece(amount) => {
                        let amount = (amount.value as f32 * adjustment_factor).round() as i32;
                        CookingIngredientAmount::Piece(CookingIngredientAmountQuarter::new(amount))
                    }
                    _ => {
                        ingredient.unit.clone()
                    }
                };
                CookingIngredient::new(ingredient.name.clone(), adjusted_amount)
            }).collect();
        dbg!(&result);
        return result
    }
}

const COOKING_INGREDIENT_AMOUNT_TBSP_TO_ML: i32 = 15;
const COOKING_INGREDIENT_AMOUNT_TSP_TO_ML: i32 = 5;
const COOKING_INGREDIENT_AMOUNT_CUP_TO_ML: i32 = 200;
const COOKING_INGREDIENT_AMOUNT_TBSP_TO_MILLIGRAM: i32 = 150;
const COOKING_INGREDIENT_AMOUNT_TSP_TO_MILLIGRAM: i32 = 50;
const COOKING_INGREDIENT_AMOUNT_CUP_TO_MILLIGRAM: i32 = 2000;

#[derive(Debug, Clone)]
pub(crate) struct CookingIngredient {
    pub(crate) name: CookingIngredientName,
    pub(crate) unit: CookingIngredientAmount,
}

impl CookingIngredient {
    pub(crate) fn new(name: CookingIngredientName, unit: CookingIngredientAmount) -> Self {
        CookingIngredient {
            name,
            unit,
        }
    }

    pub(crate) fn amount(&self) -> f32 {
        self.unit.get_value()
    }

    pub(crate) fn to_approx_unit_i18n(&self) -> SmartSpeakerI18nText {
        match self.unit {
            CookingIngredientAmount::MilliGram(amount) => {
                match self.name.to_material_property() {
                    SmartSpeakerMaterialProperty::Powder => {
                        if amount < COOKING_INGREDIENT_AMOUNT_TBSP_TO_MILLIGRAM {
                            let tsp = amount_to_approx_quarter(amount, COOKING_INGREDIENT_AMOUNT_TSP_TO_MILLIGRAM);
                            CookingIngredientAmount::Tsp(tsp).to_i18n()
                        } else {
                            let tbsp = amount_to_approx_quarter(amount, COOKING_INGREDIENT_AMOUNT_TBSP_TO_MILLIGRAM);
                            CookingIngredientAmount::Tbsp(tbsp).to_i18n()
                        }
                    }
                    SmartSpeakerMaterialProperty::Solid => {
                        let criteria = self.name.get_weight_per_one_criteria();
                        if (criteria.0..criteria.1).contains(&amount) {
                            CookingIngredientAmount::Piece(CookingIngredientAmountQuarter::new(4)).to_i18n()
                        } else {
                            let piece = amount_to_approx_quarter(amount, criteria.0);
                            CookingIngredientAmount::Piece(piece).to_i18n()
                        }
                    }
                    _ => {
                        self.unit.to_i18n()
                    }
                }
            }
            CookingIngredientAmount::MilliLiter(amount) => {
                match self.name.to_material_property() {
                    SmartSpeakerMaterialProperty::Liquid => {
                        if amount < COOKING_INGREDIENT_AMOUNT_TBSP_TO_ML {
                            let tsp = amount_to_approx_quarter(amount, COOKING_INGREDIENT_AMOUNT_TSP_TO_ML);
                            CookingIngredientAmount::Tsp(tsp).to_i18n()
                        } else {
                            let tbsp = amount_to_approx_quarter(amount, COOKING_INGREDIENT_AMOUNT_TBSP_TO_ML);
                            CookingIngredientAmount::Tbsp(tbsp).to_i18n()
                        }
                    }
                    _ => {
                        self.unit.to_i18n()
                    }
                }
            }
            _ => {
                self.unit.to_i18n()
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum CookingIngredientName {
    Salt,
    Pepper,
    Sugar,
    SoySauce,
    Sesame,
    SesameOil,
    Miso,
    Sake,
    Mirin,
    Carrot,
    Onion,
    Potato,
    Mayonnaise,
}

impl CookingIngredientName {
    pub(crate) fn to_i18n(&self) -> SmartSpeakerI18nText {
        match self {
            CookingIngredientName::Salt => {
                SmartSpeakerI18nText::new()
                    .en("salt")
                    .ja("塩")
                    .zh("盐")
                    .ko("소금")
            }
            CookingIngredientName::Pepper => {
                SmartSpeakerI18nText::new()
                    .en("pepper")
                    .ja("胡椒")
                    .zh("胡椒")
                    .ko("후추")
            }
            CookingIngredientName::Sugar => {
                SmartSpeakerI18nText::new()
                    .en("sugar")
                    .ja("砂糖")
                    .zh("糖")
                    .ko("설탕")
            }
            CookingIngredientName::SoySauce => {
                SmartSpeakerI18nText::new()
                    .en("soy sauce")
                    .ja("醤油")
                    .zh("酱油")
                    .ko("간장")
            }
            CookingIngredientName::Sesame => {
                SmartSpeakerI18nText::new()
                    .en("sesame")
                    .ja("ごま")
                    .zh("芝麻")
                    .ko("참깨")
            }
            CookingIngredientName::SesameOil => {
                SmartSpeakerI18nText::new()
                    .en("sesame oil")
                    .ja("ごま油")
                    .zh("芝麻油")
                    .ko("참기름")
            }
            CookingIngredientName::Miso => {
                SmartSpeakerI18nText::new()
                    .en("miso")
                    .ja("味噌")
                    .zh("味噌")
                    .ko("된장")
            }
            CookingIngredientName::Sake => {
                SmartSpeakerI18nText::new()
                    .en("sake")
                    .ja("酒")
                    .zh("酒")
                    .ko("술")
            }
            CookingIngredientName::Mirin => {
                SmartSpeakerI18nText::new()
                    .en("mirin")
                    .ja("みりん")
                    .zh("味醂")
                    .ko("미림")
            }
            CookingIngredientName::Carrot => {
                SmartSpeakerI18nText::new()
                    .en("carrot")
                    .ja("人参")
                    .zh("胡萝卜")
                    .ko("당근")
            }
            CookingIngredientName::Onion => {
                SmartSpeakerI18nText::new()
                    .en("onion")
                    .ja("玉ねぎ")
                    .zh("洋葱")
                    .ko("양파")
            }
            CookingIngredientName::Potato => {
                SmartSpeakerI18nText::new()
                    .en("potato")
                    .ja("じゃがいも")
                    .zh("土豆")
                    .ko("감자")
            }
            CookingIngredientName::Mayonnaise => {
                SmartSpeakerI18nText::new()
                    .en("mayonnaise")
                    .ja("マヨネーズ")
                    .zh("蛋黄酱")
                    .ko("마요네즈")
            }
        }
    }

    pub(crate) fn to_template_code(&self) -> String {
        match self {
            CookingIngredientName::Salt => {
                "salt".to_string()
            }
            CookingIngredientName::Pepper => {
                "pepper".to_string()
            }
            CookingIngredientName::Sugar => {
                "sugar".to_string()
            }
            CookingIngredientName::SoySauce => {
                "soy_sauce".to_string()
            }
            CookingIngredientName::Sesame => {
                "sesame".to_string()
            }
            CookingIngredientName::SesameOil => {
                "sesame_oil".to_string()
            }
            CookingIngredientName::Miso => {
                "miso".to_string()
            }
            CookingIngredientName::Sake => {
                "sake".to_string()
            }
            CookingIngredientName::Mirin => {
                "mirin".to_string()
            }
            CookingIngredientName::Carrot => {
                "carrot".to_string()
            }
            CookingIngredientName::Onion => {
                "onion".to_string()
            }
            CookingIngredientName::Potato => {
                "potato".to_string()
            }
            CookingIngredientName::Mayonnaise => {
                "mayonnaise".to_string()
            }
        }
    }

    pub(crate) fn to_material_property(&self) -> SmartSpeakerMaterialProperty {
        match self {
            CookingIngredientName::Salt => {
                SmartSpeakerMaterialProperty::Powder
            }
            CookingIngredientName::Pepper => {
                SmartSpeakerMaterialProperty::Powder
            }
            CookingIngredientName::Sugar => {
                SmartSpeakerMaterialProperty::Powder
            }
            CookingIngredientName::SoySauce => {
                SmartSpeakerMaterialProperty::Liquid
            }
            CookingIngredientName::Sesame => {
                SmartSpeakerMaterialProperty::Powder
            }
            CookingIngredientName::SesameOil => {
                SmartSpeakerMaterialProperty::Liquid
            }
            CookingIngredientName::Miso => {
                SmartSpeakerMaterialProperty::Liquid
            }
            CookingIngredientName::Sake => {
                SmartSpeakerMaterialProperty::Liquid
            }
            CookingIngredientName::Mirin => {
                SmartSpeakerMaterialProperty::Liquid
            }
            CookingIngredientName::Carrot => {
                SmartSpeakerMaterialProperty::Solid
            }
            CookingIngredientName::Onion => {
                SmartSpeakerMaterialProperty::Solid
            }
            CookingIngredientName::Potato => {
                SmartSpeakerMaterialProperty::Solid
            }
            CookingIngredientName::Mayonnaise => {
                SmartSpeakerMaterialProperty::Liquid
            }
        }
    }

    pub(crate) fn get_weight_per_one_criteria(&self) -> (i32, i32) {
        match self {
            CookingIngredientName::Carrot => {
                (1000, 3000)
            }
            CookingIngredientName::Onion => {
                (2000, 3000)
            }
            CookingIngredientName::Potato => {
                (700, 3000)
            }
            _ => {
                (0, 0)
            }
        }
    }

    pub(crate) fn get_weight_per_perimeter(&self, perimeter: f32) -> CookingIngredientAmount {
        // perimeter error bound: 10%
        // criteria value: (perimeter, weight(mg))
        // Returns the value in 25g increments with a margin of error.
        match self {
            CookingIngredientName::Carrot => {
                let criteria = (60.0, 1000);
                let adjusted_perimeter = perimeter * 1.05;
                let calculated_weight = (adjusted_perimeter / criteria.0) * criteria.1 as f32;
                let weight_with_error = calculated_weight * 0.95;

                let rounded_weight = if perimeter >= 50.0 {
                    ((weight_with_error / 25.0).round()) as i32 * 25
                } else {
                    ((weight_with_error / 2.5).round()) as i32 * 2
                };
                CookingIngredientAmount::MilliGram(rounded_weight)
            }
            CookingIngredientName::Potato => {
                let criteria = (60.0, 700);
                let adjusted_perimeter = perimeter * 1.05;
                let calculated_weight = (adjusted_perimeter / criteria.0) * criteria.1 as f32;
                let weight_with_error = calculated_weight * 0.95;

                let rounded_weight = if perimeter >= 50.0 {
                    ((weight_with_error / 25.0).round()) as i32 * 25
                } else {
                    ((weight_with_error / 2.5).round()) as i32 * 2
                };
                CookingIngredientAmount::MilliGram(rounded_weight)
            }
            _ => {
                CookingIngredientAmount::MilliGram(0)
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct CookingIngredientAmountQuarter {
    value: i32,
}

impl CookingIngredientAmountQuarter {
    pub(crate) fn new(value: i32) -> Self {
        CookingIngredientAmountQuarter {
            value,
        }
    }

    pub(crate) fn get_value(&self) -> f32 {
        self.value as f32 * 0.25
    }

    fn add(self, other: Self) -> Self {
        CookingIngredientAmountQuarter {
            value: self.value + other.value,
        }
    }

    fn sub(self, other: Self) -> Self {
        CookingIngredientAmountQuarter {
            value: self.value - other.value,
        }
    }

    pub(crate) fn to_i18n(&self) -> SmartSpeakerI18nText {
        match self.value {
            1 => {
                SmartSpeakerI18nText::new()
                    .en("a quarter")
                    .ja("4分の1")
                    .zh("四分之一")
                    .ko("사분의 일")
            }
            2 => {
                SmartSpeakerI18nText::new()
                    .en("half")
                    .ja("二分の一")
                    .zh("一半")
                    .ko("반")
            }
            3 => {
                SmartSpeakerI18nText::new()
                    .en("three quarters")
                    .ja("4分の3")
                    .zh("四分之三")
                    .ko("사분의 삼")
            }
            _ => {
                let full = self.value / 4;
                let rem = self.value % 4;
                if rem > 0 {
                    SmartSpeakerI18nText::new()
                        .en(&format!("{} and {}", full, CookingIngredientAmountQuarter::new(rem).to_i18n().en))
                        .ja(&format!("{}と{}", full, CookingIngredientAmountQuarter::new(rem).to_i18n().ja))
                        .zh(&format!("{}和{}", full, CookingIngredientAmountQuarter::new(rem).to_i18n().zh))
                        .ko(&format!("{}과 {}", full, CookingIngredientAmountQuarter::new(rem).to_i18n().ko))
                } else {
                    SmartSpeakerI18nText::new()
                        .en(&format!("{}", full))
                        .ja(&format!("{}", full))
                        .zh(&format!("{}", full))
                        .ko(&format!("{}", full))
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum CookingIngredientAmount {
    MilliGram(i32),
    MilliLiter(i32),
    Piece(CookingIngredientAmountQuarter),
    Tbsp(CookingIngredientAmountQuarter),
    Tsp(CookingIngredientAmountQuarter),
    Cup(CookingIngredientAmountQuarter),
}

impl CookingIngredientAmount {
    pub(crate) fn get_value(&self) -> f32 {
        match self {
            CookingIngredientAmount::MilliGram(amount) => {
                *amount as f32
            }
            CookingIngredientAmount::MilliLiter(amount) => {
                *amount as f32
            }
            CookingIngredientAmount::Piece(amount) => {
                amount.get_value()
            }
            CookingIngredientAmount::Tbsp(amount) => {
                amount.get_value()
            }
            CookingIngredientAmount::Tsp(amount) => {
                amount.get_value()
            }
            CookingIngredientAmount::Cup(amount) => {
                amount.get_value()
            }
        }
    }

    pub(crate) fn to_i18n(&self) -> SmartSpeakerI18nText {
        match self {
            CookingIngredientAmount::MilliGram(amount) => {
                SmartSpeakerI18nText::new()
                    .en(&format!("{} milligram", amount))
                    .ja(&format!("{}ミリグラム", amount))
                    .zh(&format!("{}毫克", amount))
                    .ko(&format!("{}밀리그램", amount))
            }
            CookingIngredientAmount::MilliLiter(amount) => {
                SmartSpeakerI18nText::new()
                    .en(&format!("{} milliliter", amount))
                    .ja(&format!("{}ミリリットル", amount))
                    .zh(&format!("{}毫升", amount))
                    .ko(&format!("{}밀리리터", amount))
            }
            CookingIngredientAmount::Piece(amount) => {
                SmartSpeakerI18nText::new()
                    .en(&format!("{} piece", amount.to_i18n().en))
                    .ja(&format!("{}個", amount.to_i18n().ja))
                    .zh(&format!("{}个", amount.to_i18n().zh))
                    .ko(&format!("{}개", amount.to_i18n().ko))
            }
            CookingIngredientAmount::Tbsp(amount) => {
                SmartSpeakerI18nText::new()
                    .en(&format!("{} tablespoon", amount.to_i18n().en))
                    .ja(&format!("おおさじ{}", amount.to_i18n().ja))
                    .zh(&format!("大勺{}", amount.to_i18n().zh))
                    .ko(&format!("{}큰술", amount.to_i18n().ko))
            }
            CookingIngredientAmount::Tsp(amount) => {
                SmartSpeakerI18nText::new()
                    .en(&format!("{} teaspoon", amount.to_i18n().en))
                    .ja(&format!("こさじ{}", amount.to_i18n().ja))
                    .zh(&format!("小勺{}", amount.to_i18n().zh))
                    .ko(&format!("{}작은술", amount.to_i18n().ko))
            }
            CookingIngredientAmount::Cup(amount) => {
                SmartSpeakerI18nText::new()
                    .en(&format!("{} cup", amount.to_i18n().en))
                    .ja(&format!("{}カップ", amount.to_i18n().ja))
                    .zh(&format!("{}杯", amount.to_i18n().zh))
                    .ko(&format!("{}컵", amount.to_i18n().ko))
            }
        }
    }

    pub(crate) fn to_template_code(&self) -> String {
        match self {
            CookingIngredientAmount::MilliGram(_) => {
                format!("mg")
            }
            CookingIngredientAmount::MilliLiter(_) => {
                format!("ml")
            }
            CookingIngredientAmount::Piece(_) => {
                format!("p")
            }
            CookingIngredientAmount::Tbsp(_) => {
                format!("tbsp")
            }
            CookingIngredientAmount::Tsp(_) => {
                format!("tsp")
            }
            CookingIngredientAmount::Cup(_) => {
                format!("cup")
            }
        }
    }

    pub(crate) fn add(&self, rhs: CookingIngredientAmount) -> Result<CookingIngredientAmount> {
        // add with same unit. if not return error
        match self {
            CookingIngredientAmount::MilliGram(left_amount) => {
                match rhs {
                    CookingIngredientAmount::MilliGram(right_amount) => {
                        Ok(CookingIngredientAmount::MilliGram(left_amount + right_amount))
                    }
                    CookingIngredientAmount::MilliLiter(_) => {
                        Err(anyhow!("failed to add different unit"))
                    }
                    CookingIngredientAmount::Piece(_) => {
                        Err(anyhow!("failed to add different unit"))
                    }
                    _ => {
                        Err(anyhow!("adding approx unit is not supported"))
                    }
                }
            }
            CookingIngredientAmount::MilliLiter(left_amount) => {
                match rhs {
                    CookingIngredientAmount::MilliGram(_) => {
                        Err(anyhow!("failed to add different unit"))
                    }
                    CookingIngredientAmount::MilliLiter(right_amount) => {
                        Ok(CookingIngredientAmount::MilliLiter(left_amount + right_amount))
                    }
                    CookingIngredientAmount::Piece(_) => {
                        Err(anyhow!("failed to add different unit"))
                    }
                    _ => {
                        Err(anyhow!("adding approx unit is not supported"))
                    }
                }
            }
            CookingIngredientAmount::Piece(left_amount) => {
                match rhs {
                    CookingIngredientAmount::MilliGram(_) => {
                        Err(anyhow!("failed to add different unit"))
                    }
                    CookingIngredientAmount::MilliLiter(_) => {
                        Err(anyhow!("failed to add different unit"))
                    }
                    CookingIngredientAmount::Piece(right_amount) => {
                        let new = left_amount.clone().add(right_amount);
                        Ok(CookingIngredientAmount::Piece(new))
                    }
                    _ => {
                        Err(anyhow!("adding approx unit is not supported"))
                    }
                }
            }
            _ => {
                Err(anyhow!("adding approx unit is not supported"))
            }
        }
    }

    pub(crate) fn sub(&self, rhs: CookingIngredientAmount) -> Result<CookingIngredientAmount> {
        // reduce with same unit. if not return error
        match self {
            CookingIngredientAmount::MilliGram(left_amount) => {
                match rhs {
                    CookingIngredientAmount::MilliGram(right_amount) => {
                        Ok(CookingIngredientAmount::MilliGram(left_amount - right_amount))
                    }
                    CookingIngredientAmount::MilliLiter(_) => {
                        Err(anyhow!("failed to subtract different unit"))
                    }
                    CookingIngredientAmount::Piece(_) => {
                        Err(anyhow!("failed to subtract different unit"))
                    }
                    _ => {
                        Err(anyhow!("subtracting approx unit is not supported"))
                    }
                }
            }
            CookingIngredientAmount::MilliLiter(left_amount) => {
                match rhs {
                    CookingIngredientAmount::MilliGram(_) => {
                        Err(anyhow!("failed to subtract different unit"))
                    }
                    CookingIngredientAmount::MilliLiter(right_amount) => {
                        Ok(CookingIngredientAmount::MilliLiter(left_amount - right_amount))
                    }
                    CookingIngredientAmount::Piece(_) => {
                        Err(anyhow!("failed to subtract different unit"))
                    }
                    _ => {
                        Err(anyhow!("subtracting approx unit is not supported"))
                    }
                }
            }
            CookingIngredientAmount::Piece(left_amount) => {
                match rhs {
                    CookingIngredientAmount::MilliGram(_) => {
                        Err(anyhow!("failed to subtract different unit"))
                    }
                    CookingIngredientAmount::MilliLiter(_) => {
                        Err(anyhow!("failed to subtract different unit"))
                    }
                    CookingIngredientAmount::Piece(right_amount) => {
                        Ok(CookingIngredientAmount::Piece(left_amount.clone().sub(right_amount)))
                    }
                    _ => {
                        Err(anyhow!("subtracting approx unit is not supported"))
                    }
                }
            }
            _ => {
                Err(anyhow!("subtracting approx unit is not supported"))
            }
        }
    }

    pub(crate) fn abs(&self) -> Self {
        match self {
            CookingIngredientAmount::MilliGram(amount) => {
                CookingIngredientAmount::MilliGram(amount.abs())
            }
            CookingIngredientAmount::MilliLiter(amount) => {
                CookingIngredientAmount::MilliLiter(amount.abs())
            }
            CookingIngredientAmount::Piece(amount) => {
                CookingIngredientAmount::Piece(amount.clone())
            }
            CookingIngredientAmount::Tbsp(amount) => {
                CookingIngredientAmount::Tbsp(amount.clone())
            }
            CookingIngredientAmount::Tsp(amount) => {
                CookingIngredientAmount::Tsp(amount.clone())
            }
            CookingIngredientAmount::Cup(amount) => {
                CookingIngredientAmount::Cup(amount.clone())
            }
        }
    }
}


pub(crate) struct CookingTask {
    pub(crate) menu: IntentCookingMenu,
    pub(crate) step: Vec<Box<dyn ActionExecutable>>,
    pub(crate) current_step: usize,
    pub(crate) last_revision: Option<Box<dyn Revision>>,
    pub(crate) previous_success_result: Option<SmartSpeakerTaskResult>,
    pub(crate) checkpoint: usize,
}

impl CookingTask {
    pub(crate) fn new(content: IntentContent, vision: bool) -> Result<Self> {
        match content.entities.get(0) {
            None => { Err(anyhow!("failed")) }
            Some(entity) => {
                let menu = entity.as_any().downcast_ref::<IntentCookingMenu>().unwrap().clone();
                Ok(CookingTask {
                    menu,
                    step: CookingStepBuilder::new(vision).build(menu),
                    current_step: 0,
                    last_revision: None,
                    previous_success_result: None,
                    checkpoint: 0,
                })
            }
        }
    }
}

impl Task for CookingTask {
    fn init(&mut self) -> Result<SmartSpeakerTaskResult> {
        self.try_next(Some(Box::new(IntentContent::new(IntentAction::Next, vec![]))))
    }

    fn next_index(&self) -> Option<usize> {
        if self.current_step < self.step.len() - 1 {
            Some(self.current_step + 1)
        } else {
            None
        }
    }

    fn try_next(&mut self, content: Option<Box<dyn Content>>) -> Result<SmartSpeakerTaskResult> {
        let mut current_action = self.step[self.current_step].clone();
        match content {
            None => {
                let trigger = current_action.get_action_trigger_type();
                return match trigger {
                    ActionTriggerType::None => {
                        Ok(SmartSpeakerTaskResult::with_tts(
                            trigger.to_task_type(),
                            SmartSpeakerTaskResultCode::TaskFailed(trigger.to_waiting_interaction()),
                            MachineSpeechBoilerplate::IntentFailed.to_i18n(),
                        ))
                    }
                    ActionTriggerType::Confirm => {
                        Ok(SmartSpeakerTaskResult::with_tts(
                            trigger.to_task_type(),
                            SmartSpeakerTaskResultCode::TaskFailed(trigger.to_waiting_interaction()),
                            MachineSpeechBoilerplate::IntentFailed.to_i18n(),
                        ))
                    }
                    ActionTriggerType::Vision(_) => {
                        Ok(SmartSpeakerTaskResult::with_tts(
                            trigger.to_task_type(),
                            SmartSpeakerTaskResultCode::TaskFailed(trigger.to_waiting_interaction()),
                            MachineSpeechBoilerplate::VisionFailed.to_i18n(),
                        ))
                    }
                }
            }
            Some(content) => {
                if let Some(revision) = &self.last_revision {
                    let _ = current_action.feed(content, Some(revision.clone()));
                } else {
                    let _ = current_action.feed(content, None);
                }
                let result = current_action.execute();
                match result {
                    Ok(r) => {
                        return self.handle_result(r)
                    }
                    Err(_) => {
                        return self.failed(None)
                    }
                }
            }
        }
    }

    fn handle_result(&mut self, result: SmartSpeakerTaskResult) -> Result<SmartSpeakerTaskResult> {
        match result.code {
            SmartSpeakerTaskResultCode::StepSuccess => {
                if let Ok(move_next_success) = self.internal_move_next() {
                    if move_next_success {
                        let next_action = self.step[self.current_step].clone();
                        let mut updated_result = result.clone();
                        if result.task_type == SmartSpeakerTaskType::Vision {
                            updated_result.code = SmartSpeakerTaskResultCode::TaskSuccess(WaitingInteraction::Skip);
                        } else {
                            updated_result.code = SmartSpeakerTaskResultCode::TaskSuccess(next_action.get_action_trigger_type().to_waiting_interaction());
                        }
                        self.previous_success_result = Some(updated_result.clone());
                        result.revision.and_then(|r| {
                            self.last_revision = Some(r);
                            dbg!(&self.last_revision);
                            Some(())
                        });
                        return Ok(updated_result)
                    } else {
                        let mut updated_result = result.clone();
                        updated_result.code = SmartSpeakerTaskResultCode::TaskSuccess(WaitingInteraction::Exit);
                        return Ok(updated_result)
                    }
                }
                return self.exit()
            }
            SmartSpeakerTaskResultCode::StepFailed => {
                return Ok(result)
            }
            SmartSpeakerTaskResultCode::RepeatPrevious => {
                if let Some(previous) = self.previous_success_result.clone() {
                    return Ok(previous)
                }
                Err(anyhow!("failed to repeat previous action"))
            }
            SmartSpeakerTaskResultCode::Cancelled => {
                return self.cancel()
            }
            SmartSpeakerTaskResultCode::Exit => {
                return self.exit()
            }
            _ => {
                Err(anyhow!("task execution failed"))
            }
        }
    }

    fn failed(&mut self, content: Option<Box<dyn Content>>) -> Result<SmartSpeakerTaskResult> {
        let mut current_action = self.step[self.current_step].clone();
        let trigger = current_action.get_action_trigger_type();
        match trigger {
            ActionTriggerType::Vision(_) => {
                Ok(SmartSpeakerTaskResult::with_tts(
                    trigger.to_task_type(),
                    SmartSpeakerTaskResultCode::TaskFailed(
                        current_action.get_action_trigger_type().to_waiting_interaction()),
                    MachineSpeechBoilerplate::VisionFailed.to_i18n(),
                ))
            }
            _ => {
                Ok(SmartSpeakerTaskResult::with_tts(
                    trigger.to_task_type(),
                    SmartSpeakerTaskResultCode::TaskFailed(
                        current_action.get_action_trigger_type().to_waiting_interaction()),
                    MachineSpeechBoilerplate::IntentFailed.to_i18n(),
                ))
            }
        }
    }

    fn internal_move_next(&mut self) -> Result<bool> {
        if self.current_step < self.step.len() - 1 {
            self.current_step += 1;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn internal_rollback(&mut self) -> Result<bool> {
        self.current_step = self.checkpoint;
        Ok(true)
    }

    fn exit(&self) -> Result<SmartSpeakerTaskResult> {
        Ok(SmartSpeakerTaskResult::with_tts(
            SmartSpeakerTaskType::NonVision,
            SmartSpeakerTaskResultCode::Exit,
            SmartSpeakerI18nText::new()
                .en("cooking task exit")
                .ja("料理タスクを終了します。")
                .zh("退出烹饪任务。")
                .ko("요리 작업을 종료합니다."),
        ))
    }

    fn cancel(&self) -> Result<SmartSpeakerTaskResult> {
        Ok(SmartSpeakerTaskResult::with_tts(
            SmartSpeakerTaskType::NonVision,
            SmartSpeakerTaskResultCode::Cancelled,
            SmartSpeakerI18nText::new()
                .en("cooking task cancelled")
                .ja("料理タスクをキャンセルします。")
                .zh("取消烹饪任务。")
                .ko("요리 작업을 취소합니다."),
        ))
    }
}
