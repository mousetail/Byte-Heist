use std::env;

use axum::extract::Query;
use axum::response::{IntoResponse, Redirect, Response};
use axum::Extension;
use oauth2::basic::{BasicClient, BasicTokenType};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EmptyExtraTokenFields,
    EndpointNotSet, EndpointSet, RedirectUrl, Scope, StandardTokenResponse, TokenResponse,
    TokenUrl,
};
use reqwest::StatusCode;
use serde::Deserialize;
use sqlx::prelude::FromRow;
use sqlx::{PgPool, Pool, Postgres};
use tower_sessions::Session;

use crate::discord::post_new_golfer;
use crate::error::Error;
use crate::models::account::Account;
use crate::models::InsertedId;

const GITHUB_SESSION_CSRF_KEY: &str = "GITHUB_SESSION_CSRF_TOKEN";
pub const ACCOUNT_ID_KEY: &str = "ACCOUNT_ID";

fn create_github_client(
) -> BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet> {
    let github_client_id = ClientId::new(
        env::var("GITHUB_CLIENT_ID").expect("Missing the GITHUB_CLIENT_ID environment variable."),
    );
    let github_client_secret = ClientSecret::new(
        env::var("GITHUB_CLIENT_SECRET")
            .expect("Missing the GITHUB_CLIENT_SECRET environment variable."),
    );
    let auth_url = AuthUrl::new("https://github.com/login/oauth/authorize".to_string())
        .expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
        .expect("Invalid token endpoint URL");

    // Set up the config for the Github OAuth2 process.

    BasicClient::new(github_client_id)
        .set_client_secret(github_client_secret)
        .set_auth_uri(auth_url)
        .set_token_uri(token_url)
        .set_redirect_uri(
            RedirectUrl::new(format!(
                "{}/callback/github",
                env::var("YQ_PUBLIC_URL").expect("Missing the YQ_PUBLIC_URL environment variable")
            ))
            .expect("Invalid redirect URL"),
        )
}

#[axum::debug_handler]
pub async fn github_login(session: Session) -> Redirect {
    let client = create_github_client();

    // Generate the authorization URL to which we'll redirect the user.
    let (authorize_url, csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("user:read".to_string()))
        .url();

    session
        .insert(GITHUB_SESSION_CSRF_KEY, csrf_state)
        .await
        .unwrap();

    return Redirect::temporary(authorize_url.as_str());
}

#[derive(Deserialize)]
pub struct GithubResponse {
    code: AuthorizationCode,
    state: CsrfToken,
}

#[derive(Deserialize, Debug)]
pub struct GithubUser {
    login: String,
    id: i64,
    avatar_url: String,
}

pub async fn github_callback(
    session: Session,
    Extension(pool): Extension<PgPool>,
    Query(token): Query<GithubResponse>,
) -> Result<Response, Error> {
    let client = create_github_client();

    let http_client = reqwest::ClientBuilder::new()
        // Following redirects opens the client up to SSRF vulnerabilities.
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Client should build");

    let GithubResponse { code, state } = token;

    if !session
        .get(GITHUB_SESSION_CSRF_KEY)
        .await
        .ok()
        .and_then(|b| b)
        .is_some_and(|d: CsrfToken| d.secret() == state.secret())
    {
        return Err(Error::Oauth(crate::error::OauthError::CsrfValidation));
    }

    let token_res = client
        .exchange_code(code)
        .request_async(&http_client)
        .await
        .map_err(|_| Error::Oauth(crate::error::OauthError::TokenExchange))?;

    let token = token_res.access_token();

    let response = http_client
        .get("https://api.github.com/user")
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "Rust-Reqwest (YQ)")
        .bearer_auth(token.secret())
        .send()
        .await
        .map_err(|_k| Error::Oauth(crate::error::OauthError::UserInfoFetch))?;

    if response.status().is_success() {
        let mut user_info: GithubUser = response
            .json()
            .await
            .map_err(|_k| Error::Oauth(crate::error::OauthError::Deserialization))?;

        if user_info.avatar_url.len() > 255 {
            // TODO: Figure out why this happens
            user_info.avatar_url = format!(
                "https://avatars.githubusercontent.com/u/{}?v=4",
                user_info.id
            );
        }

        insert_user(&pool, &user_info, &token_res, &session).await?;
        Ok(Redirect::temporary("/").into_response())
    } else {
        let data = response.bytes().await.unwrap();
        Ok((
            StatusCode::INTERNAL_SERVER_ERROR,
            String::from_utf8_lossy(&data).to_string(),
        )
            .into_response())
    }
}

#[derive(FromRow)]
struct UserQueryResponse {
    id: i32,
    account: i32,
}

async fn insert_user(
    pool: &Pool<Postgres>,
    github_user: &GithubUser,
    token: &StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
    session: &Session,
) -> Result<(), Error> {
    let sql = "SELECT id, account FROM account_oauth_codes WHERE id_on_provider=$1";

    let user: Option<UserQueryResponse> = sqlx::query_as::<_, UserQueryResponse>(sql)
        .bind(github_user.id)
        .fetch_optional(pool)
        .await
        .map_err(Error::Database)?;

    if let Some(user) = user {
        let sql: &str =
            "UPDATE account_oauth_codes SET access_token=$1, refresh_token=$2 WHERE id=$3";

        sqlx::query_as::<_, UserQueryResponse>(sql)
            .bind(token.access_token().secret())
            .bind(
                token
                    .refresh_token()
                    .map(|d| d.secret().as_str())
                    .unwrap_or(""),
            )
            .bind(user.id)
            .fetch_optional(pool)
            .await
            .map_err(Error::Database)?;

        session.insert(ACCOUNT_ID_KEY, user.account).await.unwrap();

        Ok(())
    } else {
        let sql: &str = "INSERT INTO accounts(username, avatar) VALUES ($1, $2) RETURNING id";

        let new_user_id: InsertedId = sqlx::query_as(sql)
            .bind(&github_user.login)
            .bind(&github_user.avatar_url)
            .fetch_one(pool)
            .await
            .map_err(Error::Database)?;

        let sql: &str =
            "INSERT INTO account_oauth_codes(account, access_token, refresh_token, id_on_provider) VALUES
        ($1, $2, $3, $4)";

        sqlx::query(sql)
            .bind(new_user_id.0)
            .bind(token.access_token().secret())
            .bind(
                token
                    .refresh_token()
                    .map(|d| d.secret().as_str())
                    .unwrap_or(""),
            )
            .bind(github_user.id)
            .execute(pool)
            .await
            .map_err(Error::Database)?;

        session.insert(ACCOUNT_ID_KEY, new_user_id.0).await.unwrap();

        tokio::spawn(post_new_golfer(Account {
            id: new_user_id.0,
            username: github_user.login.clone(),
            avatar: github_user.avatar_url.clone(),
            preferred_language: "python".to_owned(),
            admin: false,
        }));

        Ok(())
    }
}
