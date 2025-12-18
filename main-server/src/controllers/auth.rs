use std::borrow::Cow;
use std::env;

use axum::Extension;
use axum::extract::Query;
use axum::response::Redirect;
use oauth2::basic::{BasicClient, BasicTokenType};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EmptyExtraTokenFields,
    EndpointNotSet, EndpointSet, RedirectUrl, Scope, StandardTokenResponse, TokenResponse,
    TokenUrl,
};
use serde::Deserialize;
use sqlx::prelude::FromRow;
use sqlx::{Executor, PgPool, Pool, Postgres};
use tower_sessions::Session;

use crate::discord::DiscordEventSender;
use crate::error::Error;

const GITHUB_SESSION_CSRF_KEY: &str = "GITHUB_SESSION_CSRF_TOKEN";
pub const ACCOUNT_ID_KEY: &str = "ACCOUNT_ID";

fn create_github_client()
-> BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet> {
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
                env::var("BYTE_HEIST_PUBLIC_URL")
                    .expect("Missing the BYTE_HEIST_PUBLIC_URL environment variable")
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

    Redirect::temporary(authorize_url.as_str())
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
    Extension(bot): Extension<DiscordEventSender>,
    Query(token): Query<GithubResponse>,
) -> Result<(), Error> {
    let client = create_github_client();

    let http_client = reqwest::ClientBuilder::new()
        // Following redirects opens the client up to SSRF vulnerabilities.
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Client should build");

    let GithubResponse { code, state } = token;

    if session
        .get(GITHUB_SESSION_CSRF_KEY)
        .await
        .ok()
        .and_then(|b| b)
        .is_none_or(|d: CsrfToken| d.secret() != state.secret())
    {
        return Err(Error::Oauth(crate::error::OauthError::CsrfValidation));
    }

    let referrer: Option<String> = session.get("referrer").await.unwrap_or(None);

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
        .header(
            "User-Agent",
            "Rust-Reqwest (Byte Heist https://byte-heist.com)",
        )
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

        update_or_insert_user(&pool, &user_info, bot, &token_res, &session, referrer).await?;
        Err(Error::Redirect(
            crate::error::RedirectType::TemporaryGet,
            Cow::Borrowed("/"),
        ))
    } else {
        // let data = response.bytes().await.unwrap();
        Err(Error::ServerError)
        //     StatusCode::INTERNAL_SERVER_ERROR,
        //     String::from_utf8_lossy(&data).to_string(),
        // )
        //     .into_response())
    }
}

#[derive(FromRow)]
struct UserQueryResponse {
    id: i32,
    account: i32,
}

async fn update_or_insert_user(
    pool: &Pool<Postgres>,
    github_user: &GithubUser,
    bot: DiscordEventSender,
    token: &StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
    session: &Session,
    referrer: Option<String>,
) -> Result<(), Error> {
    let sql = "SELECT id, account FROM account_oauth_codes WHERE id_on_provider=$1";

    let user: Option<UserQueryResponse> = sqlx::query_as::<_, UserQueryResponse>(sql)
        .bind(github_user.id)
        .fetch_optional(pool)
        .await
        .map_err(Error::Database)?;

    if let Some(user) = user {
        update_account_oauth_codes(
            pool,
            user.id,
            token.access_token().secret(),
            token
                .refresh_token()
                .map(|d| d.secret().as_str())
                .unwrap_or(""),
        )
        .await
        .map_err(Error::Database)?;

        session.insert(ACCOUNT_ID_KEY, user.account).await.unwrap();

        Ok(())
    } else {
        let mut transaction = pool.begin().await.map_err(Error::Database)?;

        let new_user_id = create_account(
            &mut *transaction,
            &github_user.login,
            &github_user.avatar_url,
            referrer.as_deref(),
        )
        .await
        .map_err(Error::Database)?;

        create_account_oauth_codes(
            &mut *transaction,
            new_user_id,
            token.access_token().secret(),
            token
                .refresh_token()
                .map(|d| d.secret().as_str())
                .unwrap_or(""),
            github_user.id,
        )
        .await
        .map_err(Error::Database)?;

        session.insert(ACCOUNT_ID_KEY, new_user_id).await.unwrap();

        bot.send(crate::discord::DiscordEvent::NewGolfer {
            user_id: new_user_id,
            referrer,
        })
        .await
        .unwrap();

        transaction.commit().await.map_err(Error::Database)?;

        Ok(())
    }
}

async fn update_account_oauth_codes(
    pool: &PgPool,
    id: i32,
    access_token: &str,
    refresh_token: &str,
) -> Result<(), sqlx::Error> {
    return sqlx::query!(
        "UPDATE account_oauth_codes SET access_token=$1, refresh_token=$2 WHERE id=$3",
        access_token,
        refresh_token,
        id
    )
    .execute(pool)
    .await
    .map(|_| ());
}

async fn create_account_oauth_codes<'c>(
    pool: impl Executor<'c, Database = Postgres>,
    id: i32,
    access_token: &str,
    refresh_token: &str,
    id_on_provider: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            INSERT INTO
                account_oauth_codes(
                    account,
                    access_token,
                    refresh_token,
                    id_on_provider
                ) VALUES
                (
                    $1,
                    $2,
                    $3,
                    $4
                )
        "#,
        id,
        access_token,
        refresh_token,
        id_on_provider
    )
    .execute(pool)
    .await
    .map(|_| ())
}

async fn create_account<'c>(
    pool: impl Executor<'c, Database = Postgres>,
    username: &str,
    avatar: &str,
    referrer: Option<&str>,
) -> Result<i32, sqlx::Error> {
    sqlx::query_scalar!(
        "INSERT INTO accounts(username, avatar, referrer) VALUES ($1, $2, $3) RETURNING id",
        username,
        avatar,
        referrer.map(|i| &i[..128])
    )
    .fetch_one(pool)
    .await
}
