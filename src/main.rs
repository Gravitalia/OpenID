use warp::{Filter, reject::Reject};
mod router;
mod helpers;
mod database;

#[derive(Debug)]
struct InvalidQuery;
impl Reject for InvalidQuery {}

#[derive(Debug)]
struct UnknownError;
impl Reject for UnknownError {}

async fn middleware(token: Option<String>, fallback: String) -> String {
    if token.is_some() && fallback == *"@me" {
        match helpers::get_jwt(token.unwrap()) {
            Ok(data) => {
                data.claims.sub
            },
            Err(_) => "Invalid".to_string()
        }
    } else if fallback == *"@me" {
        "Invalid".to_string()
    } else {
        fallback
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let routes = warp::path("create").and(warp::post()).and(warp::body::json()).and(warp::header("sec")).and(warp::header("cf-turnstile-token")).and_then(|body: router::model::Create, finger: String, _cf_token: String| async {
        match router::create::create(body, finger).await {
            Ok(r) => {
                Ok(r)
            },
            Err(_) => {
                Err(warp::reject::custom(UnknownError))
            }
        }
    })
    .or(warp::path!("users" / String).and(warp::get()).and(warp::header::optional::<String>("authorization")).and_then(|id: String, token: Option<String>| async {
        // Lets's check Sec header later
        let middelware_res: String = middleware(token, id).await;
        if middelware_res != *"Invalid" {
            Ok(router::users::get(middelware_res.to_lowercase()).await)
        } else {
            Err(warp::reject::custom(InvalidQuery))
        }
    }))
    .or(warp::path("login").and(warp::post()).and(warp::body::json()).and(warp::header("sec")).and(warp::header("cf-turnstile-token")).and_then(|body: router::model::Login, finger: String, _cf_token: String| async {
        match router::login::login(body, finger).await {
            Ok(r) => {
                Ok(r)
            },
            Err(_) => {
                Err(warp::reject::custom(UnknownError))
            }
        }
    }))
    .or(warp::path!("users" / "@me").and(warp::patch()).and(warp::body::json()).and(warp::header("authorization")).and_then(|body: router::model::UserPatch, token: String| async {
        let middelware_res: String = middleware(Some(token), "@me".to_string()).await;
        if middelware_res != *"Invalid" {
            Ok(router::users::patch(body, middelware_res).await)
        } else {
            Err(warp::reject::custom(UnknownError))
        }
    }));

    database::cassandra::init().await;
    database::cassandra::tables().await;
    database::mem::init();
    helpers::init();

    warp::serve(warp::any().and(warp::options()).map(|| "OK").or(routes))
    .run((
        [127, 0, 0, 1],
        dotenv::var("PORT").expect("Missing env `PORT`").parse::<u16>().unwrap(),
    ))
    .await;
}