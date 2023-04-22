use std::{collections::HashMap, error::Error};

use serde::{Serialize, Deserialize};

use crate::{ui::widget::{button::UIButtonStyle, checkbox::UICheckBoxStyle}, uid::UID, registry::asset::Asset};

#[derive(Debug)]
pub enum UIStyleSheetError {
    DuplicatedButtonStyle { uid: UID },
    DuplicatedCheckboxStyle { uid: UID },
    ButtonStyleNotFound { uid: UID },
    CheckboxStyleNotFound { uid: UID },
}

impl Error for UIStyleSheetError {}

impl std::fmt::Display for UIStyleSheetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UIStyleSheetError::DuplicatedButtonStyle { uid } => write!(f, "Button style already exists: {}", uid),
            UIStyleSheetError::DuplicatedCheckboxStyle { uid } => write!(f, "Checkbox style already exists: {}", uid),
            UIStyleSheetError::ButtonStyleNotFound { uid } => write!(f, "Button style not found: {}", uid),
            UIStyleSheetError::CheckboxStyleNotFound { uid } => write!(f, "Checkbox style not found: {}", uid),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UIStyleSheet {
    pub(crate) buttons: HashMap<UID, UIButtonStyle>,
    pub(crate) checkboxes: HashMap<UID, UICheckBoxStyle>,
}

impl UIStyleSheet {

    pub const NAME: &'static str = "ui_stylesheet";
    pub const UID: UID = UID::new(UIStyleSheet::NAME);

    pub(crate) fn merge(&mut self, other: &Self) -> Result<(), UIStyleSheetError> {
        for (uid, style) in &other.buttons {
            if self.buttons.contains_key(uid) {
                return Err(UIStyleSheetError::DuplicatedButtonStyle { uid: *uid });
            }
            self.buttons.insert(*uid, style.clone());
        }
        Ok(())
    }

    pub fn empty() -> Self {
        Self {
            buttons: Default::default(),
            checkboxes: Default::default(),
        }
    }

    pub fn add_button_style(&mut self, name: &str, style: UIButtonStyle) -> Result<UID, UIStyleSheetError> {
        let uid = UID::new(name);
        if self.buttons.contains_key(&uid) {
            return Err(UIStyleSheetError::DuplicatedButtonStyle { uid });
        }
        self.buttons.insert(uid, style);
        Ok(uid)
    }

    pub fn add_checkbox_style(&mut self, name: &str, style: UICheckBoxStyle) -> Result<UID, UIStyleSheetError> {
        let uid = UID::new(name);
        if self.checkboxes.contains_key(&uid) {
            return Err(UIStyleSheetError::DuplicatedCheckboxStyle { uid });
        }
        self.checkboxes.insert(uid, style);
        Ok(uid)
    }
}

impl Default for UIStyleSheet {
    fn default() -> Self {
        let mut stylesheet = Self::empty();
        stylesheet.add_button_style(UIButtonStyle::DEFAULT, UIButtonStyle::default()).unwrap();
        stylesheet.add_checkbox_style(UICheckBoxStyle::DEFAULT, UICheckBoxStyle::default()).unwrap();
        stylesheet
    }
}

impl Asset for UIStyleSheet {}