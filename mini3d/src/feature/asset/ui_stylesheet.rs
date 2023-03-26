use std::collections::HashMap;

use anyhow::{anyhow, Result};
use serde::{Serialize, Deserialize};

use crate::{ui::widget::{button::UIButtonStyle, checkbox::UICheckBoxStyle}, uid::UID, registry::asset::Asset};

#[derive(Serialize, Deserialize, Clone)]
pub struct UIStyleSheet {
    pub(crate) buttons: HashMap<UID, UIButtonStyle>,
    pub(crate) checkboxes: HashMap<UID, UICheckBoxStyle>,
}

impl UIStyleSheet {

    pub const NAME: &'static str = "ui_stylesheet";
    pub const UID: UID = UID::new(UIStyleSheet::NAME);

    pub(crate) fn merge(&mut self, other: &Self) -> Result<()> {
        for (uid, style) in &other.buttons {
            if self.buttons.contains_key(uid) {
                return Err(anyhow!("Button style with name '{}' already exists", uid));
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

    pub fn add_button_style(&mut self, name: &str, style: UIButtonStyle) -> Result<UID> {
        let uid = UID::new(name);
        if self.buttons.contains_key(&uid) {
            return Err(anyhow!("Button style with name '{}' already exists", name));
        }
        self.buttons.insert(uid, style);
        Ok(uid)
    }

    pub fn add_checkbox_style(&mut self, name: &str, style: UICheckBoxStyle) -> Result<UID> {
        let uid = UID::new(name);
        if self.checkboxes.contains_key(&uid) {
            return Err(anyhow!("Checkbox style with name '{}' already exists", name));
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