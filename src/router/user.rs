//! Get and update user data.

use axum::extract::{Path, State};
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::router::ServerError;
use crate::user::User;
use crate::AppState;

const ACTIVITY_STREAM: &str = "https://www.w3.org/ns/activitystreams";
const W3C_SECURITY: &str = "https://w3id.org/security/v1";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum ActivityType {
    Add,
    Application,
    Article,
    Collection,
    Create,
    Image,
    Like,
    Link,
    Note,
    Object,
    OrderedCollection,
    Person,
    Place,
    Point,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    #[serde(rename = "@context")]
    context: Vec<String>,
    r#type: ActivityType,
    id: String,
    #[serde(rename = "preferredUsername")]
    username: String,
    name: String,
    url: String,
    summary: String,
    published: String,
    public_key: Vec<Key>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Key {
    id: String,
    owner: String,
    public_key_pem: String,
}

pub async fn get(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<Response>, ServerError> {
    let user = User::default()
        .with_vanity(user_id)
        .get(&state.db.postgres)
        .await?;
    let url = if let Ok(mut url) = url::Url::parse(&state.config.url) {
        url.set_path(&format!("/users/{}", user.vanity));
        format!("{}://{}/users/{}",
            url.scheme(),
            url.host().map(|u| u.to_string()).unwrap_or_default(),
            user.vanity,
        )
    } else {
        user.vanity.clone()
    };

    Ok(Json(Response {
        context: vec![
            ACTIVITY_STREAM.to_string(),
            W3C_SECURITY.to_string(),
        ],
        r#type: ActivityType::Person,
        id: user.vanity.clone(),
        username: user.username.clone(),
        name: user.username,
        url, 
        summary: String::default(),
        published: user.created_at.to_string(),
        public_key: Vec::new(),
    }))
}
