use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, SaltString},
    Argon2, Params, Version,
};
use axum::{extract::State, http::StatusCode, Json};
use regex_lite::Regex;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

use super::{ServerError, Valid};
use crate::{database::Database, user::User};

fn validate_vanity(vanity: &str) -> Result<(), ValidationError> {
    if !Regex::new(r"[A-Za-z0-9\-\.\_\~\!\$\&\'\(\)\*\+\,\;\=](?:[A-Za-z0-9\-\.\_\~\!\$\&\'\(\)\*\+\,\;\=]|(?:%[0-9A-Fa-f]{2}))$")
            .map_err(|_| ValidationError::new("wtf_regex"))?
            .is_match(vanity) {
        return Err(ValidationError::new("alphanumerical"));
    }

    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct Body {
    #[validate(
        length(min = 2, max = 15),
        custom(function = "validate_vanity", message = "Vanity must be alphanumeric.")
    )]
    vanity: String,
    #[validate(email(message = "Email must be formated."))]
    email: String,
    #[validate(length(min = 8, message = "Password must contain at least 8 characters."))]
    password: String,
    _captcha: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Response {
    user: User,
    token: String,
}

pub async fn create(
    State(db): State<Database>,
    Valid(body): Valid<Body>,
) -> Result<(StatusCode, Json<Response>), ServerError> {
    let email = crate::crypto::email_encryption(body.email);

    let password = {
        let salt = SaltString::generate(&mut OsRng);
        let params = Params::new(Params::DEFAULT_M_COST * 4, 6, Params::DEFAULT_P_COST, None)
            .map_err(|err| ServerError::Internal(err.to_string()))?;
        let argon2 = Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, params);

        let password_hash = argon2
            .hash_password(body.password.as_bytes(), &salt)
            .map_err(|err| ServerError::Internal(err.to_string()))?
            .to_string();
        let hash = PasswordHash::new(&password_hash)
            .map_err(|err| ServerError::Internal(err.to_string()))?;

        hash.to_string()
    };

    sqlx::query!(
        r#"INSERT INTO "users" (vanity, username, email, password) values ($1, $2, $3, $4)"#,
        body.vanity.to_lowercase(),
        body.vanity,
        email,
        password
    )
    .execute(&db.postgres)
    .await?;

    let user = User::default()
        .with_vanity(body.vanity.to_lowercase())
        .get(&db.postgres)
        .await?;
    let token = user.generate_token(&db.postgres).await?;

    Ok((StatusCode::CREATED, Json(Response { user, token })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    use axum::{
        body::Body as RequestBody,
        http::{self, Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use sqlx::{Pool, Postgres};
    use tower::ServiceExt;

    #[sqlx::test]
    async fn test_create_handler(pool: Pool<Postgres>) {
        let state = AppState {
            db: Database { postgres: pool },
            config: status::Configuration::default(),
        };
        let app = app(state);

        let body = Body {
            vanity: "user".into(),
            email: "test@gravitalia.com".into(),
            password: "Password1234".into(),
            _captcha: None,
        };
        let body = serde_json::to_string(&body).unwrap();
        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/create")
                    .header(http::header::CONTENT_TYPE, "application/json")
                    .body(RequestBody::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: Response = serde_json::from_slice(&body).unwrap();
        assert!(body.token.is_ascii());
        assert_eq!(body.user.vanity, "user");
    }
}
