use regex::Regex;
use warp::reply::{WithStatus, Json};
use sha256::digest;
use crate::database::cassandra::query;

pub async fn create(body: super::model::Create, finger: String) -> WithStatus<Json> {
    // Email verification
    if !Regex::new(r"^([a-zA-Z0-9_\-\.]+)@([a-zA-Z0-9_\-\.]+)\.([a-zA-Z]{2,7})$").unwrap().is_match(&body.email) {
        return super::err("Invalid email".to_string());
    }
    // Password checking [Security]
    if !Regex::new(r"([0-9|*|]|[$&+,:;=?@#|'<>.^*()%!-])+").unwrap().is_match(&body.password) {
        return super::err("Invalid password".to_string());
    }
    // Vanity verification
    if !Regex::new(r"[A-z|0-9|_]{3,16}").unwrap().is_match(&body.vanity) {
        return super::err("Invalid vanity".to_string());
    }
    // Username checking
    if body.username.len() >= 16 {
		return super::err("Invalid username".to_string());
	}

    if !&query("SELECT vanity FROM accounts.users WHERE email = ?", vec![digest(&*body.email)]).await.response_body().unwrap().as_cols().unwrap().rows_content.is_empty() {
        warp::reply::with_status(warp::reply::json(
            &super::model::Error{
                error: true,
                message: "Invalid email".to_string()
            }
        ),
        warp::http::StatusCode::BAD_REQUEST)
    } else if !&query("SELECT vanity FROM accounts.users WHERE vanity = ?", vec![digest(&*body.vanity)]).await.response_body().unwrap().as_cols().unwrap().rows_content.is_empty() {
            warp::reply::with_status(warp::reply::json(
                &super::model::Error{
                    error: true,
                    message: "Invalid vanity".to_string()
                }
            ),
            warp::http::StatusCode::BAD_REQUEST)
    } else {
        // Phone verification
        let mut phone: Option<String> = None;
        if body.phone.is_some() {
            if !Regex::new(r"^\s*(?:\+?(\d{1,3}))?[-. (]*(\d{3})[-. )]*(\d{3})[-. ]*(\d{4})(?: *x(\d+))?\s*$").unwrap().is_match(body.phone.as_ref().unwrap()) {
                return super::err("Invalid phone".to_string());
            } else {
                phone = Some(crate::helpers::encrypt(body.phone.unwrap().as_bytes()));
            }
        }
        // Birthdate verification
        let mut birth: Option<String> = None;
        if body.birthdate.is_some() {
            if !Regex::new(r"^\d{4}-(0[1-9]|1[0-2])-(0[1-9]|[12][0-9]|3[01])$").unwrap().is_match(body.birthdate.as_ref().unwrap()) {
                return super::err("Invalid birthdate".to_string());
            } else {
                birth = Some(crate::helpers::encrypt(body.birthdate.unwrap().as_bytes()));
            }
        }

        crate::database::cassandra::create_user(&body.vanity.to_lowercase(), digest(body.email), body.username, crate::helpers::hash(body.password.as_ref()), phone, birth).await;

        warp::reply::with_status(warp::reply::json(
            &super::model::CreateResponse{
                token: crate::helpers::create_jwt(body.vanity.to_lowercase(), Some(digest(&*finger)), Some(crate::database::cassandra::create_security(body.vanity.to_lowercase(), crate::router::model::SecurityCode::Jwt as u8, finger, None, None).await.to_string())).await
            }
        ),
        warp::http::StatusCode::CREATED)
    }
}