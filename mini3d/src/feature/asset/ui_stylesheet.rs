use std::collections::HashMap;

use mini3d_derive::{Asset, Error};

use crate::{
    ui::widget::{button::UIButtonStyle, checkbox::UICheckBoxStyle},
    uid::UID,
};

#[derive(Debug, Error)]
pub enum UIStyleSheetError {
    #[error("Duplicated button style: {uid}")]
    DuplicatedButtonStyle { uid: UID },
    #[error("Duplicated checkbox style: {uid}")]
    DuplicatedCheckboxStyle { uid: UID },
    #[error("Button style not found: {uid}")]
    ButtonStyleNotFound { uid: UID },
    #[error("Checkbox style not found: {uid}")]
    CheckboxStyleNotFound { uid: UID },
}

#[derive(Asset, Clone)]
pub struct UIStyleSheet {
    pub(crate) buttons: HashMap<UID, UIButtonStyle>,
    pub(crate) checkboxes: HashMap<UID, UICheckBoxStyle>,
}

impl UIStyleSheet {
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

    pub fn add_button_style(
        &mut self,
        name: &str,
        style: UIButtonStyle,
    ) -> Result<UID, UIStyleSheetError> {
        let uid = UID::new(name);
        if self.buttons.contains_key(&uid) {
            return Err(UIStyleSheetError::DuplicatedButtonStyle { uid });
        }
        self.buttons.insert(uid, style);
        Ok(uid)
    }

    pub fn add_checkbox_style(
        &mut self,
        name: &str,
        style: UICheckBoxStyle,
    ) -> Result<UID, UIStyleSheetError> {
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
        stylesheet
            .add_button_style(UIButtonStyle::DEFAULT, UIButtonStyle::default())
            .unwrap();
        stylesheet
            .add_checkbox_style(UICheckBoxStyle::DEFAULT, UICheckBoxStyle::default())
            .unwrap();
        stylesheet
    }
}
