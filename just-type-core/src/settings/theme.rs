use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, PartialEq, Eq, Default, Clone, Copy, Debug)]
pub enum Theme {
    #[default]
    Light,
    Dark,
}
