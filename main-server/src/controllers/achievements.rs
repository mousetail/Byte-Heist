use axum::extract::Path;
use serde::Serialize;
use strum::VariantArray;

use crate::{achievements::AchievementType, error::Error};

#[derive(Serialize)]
pub struct Achievement {
    icon: String,
}

pub async fn list_achievements(_path: Option<Path<String>>) -> Result<Vec<Achievement>, Error> {
    Ok(AchievementType::VARIANTS
        .into_iter()
        .map(|i| Achievement { icon: i.get_icon() })
        .collect())
}
