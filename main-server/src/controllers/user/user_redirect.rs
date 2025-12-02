use std::borrow::Cow;

use axum::{Extension, extract::Path};
use common::slug::Slug;
use sqlx::{PgPool, query_scalar};

use crate::error::Error;

pub async fn get_username(pool: &PgPool, user_id: i32) -> Result<String, sqlx::Error> {
    query_scalar!(
        "
        SELECT username
        FROM accounts
        WHERE id=$1
        ",
        user_id
    )
    .fetch_one(pool)
    .await
}

pub async fn redirect_to_user_page(
    Path(user_id): Path<i32>,
    Extension(pool): Extension<PgPool>,
) -> Result<(), crate::error::Error> {
    let username = get_username(&pool, user_id)
        .await
        .map_err(Error::Database)?;

    Err(Error::Redirect(
        crate::error::RedirectType::Permanent,
        Cow::Owned(format!("/user/{user_id}/{}", Slug(&username))),
    ))
}
