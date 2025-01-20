use crate::chat::constant::AVAILABLE_MODELS;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub enum UsageCheck {
    None,
    Default,
    All,
    Custom(Vec<&'static str>),
}

impl Default for UsageCheck {
    fn default() -> Self {
        Self::Default
    }
}

impl Serialize for UsageCheck {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("UsageCheck", 1)?;
        match self {
            UsageCheck::None => {
                state.serialize_field("type", "none")?;
            }
            UsageCheck::Default => {
                state.serialize_field("type", "default")?;
            }
            UsageCheck::All => {
                state.serialize_field("type", "all")?;
            }
            UsageCheck::Custom(models) => {
                state.serialize_field("type", "list")?;
                state.serialize_field("content", &models.join(","))?;
            }
        }
        state.end()
    }
}

impl<'de> Deserialize<'de> for UsageCheck {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(tag = "type", content = "content")]
        enum UsageCheckHelper {
            #[serde(rename = "none")]
            None,
            #[serde(rename = "default")]
            Default,
            #[serde(rename = "all")]
            All,
            #[serde(rename = "list")]
            Custom(String),
        }

        let helper = UsageCheckHelper::deserialize(deserializer)?;
        Ok(match helper {
            UsageCheckHelper::None => UsageCheck::None,
            UsageCheckHelper::Default => UsageCheck::Default,
            UsageCheckHelper::All => UsageCheck::All,
            UsageCheckHelper::Custom(list) => {
                if list.is_empty() {
                    return Ok(UsageCheck::None);
                }

                let models: Vec<&'static str> = list
                    .split(',')
                    .filter_map(|model| {
                        let model = model.trim();
                        AVAILABLE_MODELS
                            .iter()
                            .find(|m| m.id == model)
                            .map(|m| m.id)
                    })
                    .collect();

                if models.is_empty() {
                    UsageCheck::None
                } else {
                    UsageCheck::Custom(models)
                }
            }
        })
    }
}
