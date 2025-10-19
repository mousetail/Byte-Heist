use std::borrow::Cow;

use axum::{Extension, extract::Path};
use serde::Deserialize;
use sqlx::{PgPool, query_scalar};

use crate::{error::Error, models::account::Account, tera_utils::auto_input::AutoInput};

#[derive(Deserialize)]
pub struct NewReaction {
    comment_id: i32,
    is_upvote: bool,
}

impl NewReaction {
    async fn submit(&self, author: i32, pool: &PgPool) -> Result<i32, sqlx::Error> {
        query_scalar!(
            "
            INSERT INTO challenge_comment_votes(
                author,
                comment,
                is_upvote
            )
            VALUES ($1, $2, $3)
            ON CONFLICT(author, comment) DO UPDATE SET is_upvote=$3
            RETURNING id
            ",
            author,
            self.comment_id,
            self.is_upvote
        )
        .fetch_one(pool)
        .await
    }
}

pub async fn post_reaction(
    Path((id, slug)): Path<(i32, String)>,
    account: Account,
    Extension(pool): Extension<PgPool>,
    AutoInput(reaction): AutoInput<NewReaction>,
) -> Result<(), Error> {
    reaction
        .submit(account.id, &pool)
        .await
        .map_err(Error::Database)?;

    Err(Error::Redirect(Cow::Owned(format!(
        "/challenge/{id}/{slug}/view"
    ))))
}
