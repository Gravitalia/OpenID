use scylla::transport::errors::QueryError;
use scylla::{Session, SessionBuilder};
use std::sync::Arc;

// Define constants for table creation and queries
const CREATE_USERS_TABLE: &str = "CREATE TABLE IF NOT EXISTS accounts.users ( vanity TEXT, email TEXT, username TEXT, avatar TEXT, banner TEXT, bio TEXT, verified BOOLEAN, flags INT, phone TEXT, password TEXT, birthdate TEXT, deleted BOOLEAN, mfa_code TEXT, expire_at TIMESTAMP, PRIMARY KEY (vanity) );";
const CREATE_BOTS_TABLE: &str = "CREATE TABLE IF NOT EXISTS accounts.bots ( id TEXT, user_id TEXT, client_secret TEXT, username TEXT, avatar TEXT, bio TEXT, redirect_url SET<TEXT>, flags INT, deleted BOOLEAN, PRIMARY KEY (id) );";
const CREATE_OAUTH_TABLE: &str = "CREATE TABLE IF NOT EXISTS accounts.oauth ( id TEXT, user_id TEXT, bot_id TEXT, scope SET<TEXT>, deleted BOOLEAN, PRIMARY KEY (id) );";
const CREATE_TOKENS_TABLE: &str = "CREATE TABLE IF NOT EXISTS accounts.tokens ( id TEXT, user_id TEXT, ip TEXT, date TIMESTAMP, expire_at TIMESTAMP, deleted BOOLEAN, PRIMARY KEY (id) );";
const CREATE_SALTS_TABLE: &str = "CREATE TABLE IF NOT EXISTS accounts.salts ( id TEXT, salt TEXT, PRIMARY KEY (id) );";
const CREATE_USERS_INDEX_EMAIL: &str =
    "CREATE INDEX IF NOT EXISTS ON accounts.users ( email );";
const CREATE_USERS_INDEX_EXPIRE_AT: &str =
    "CREATE INDEX IF NOT EXISTS ON accounts.users ( expire_at );";
const CREATE_OAUTH_INDEX_USER_ID: &str =
    "CREATE INDEX IF NOT EXISTS ON accounts.oauth ( user_id );";
const CREATE_TOKENS_INDEX_USER_ID: &str =
    "CREATE INDEX IF NOT EXISTS ON accounts.tokens ( user_id );";
const CREATE_USER: &str = "INSERT INTO accounts.users ( vanity, email, username, password, phone, birthdate, avatar, bio, flags, deleted, verified, expire_at ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, 0, false, false, 0);";
const CREATE_OAUTH: &str = "INSERT INTO accounts.oauth ( id, user_id, bot_id, scope, deleted ) VALUES (?, ?, ?, ?, ?)";
const CREATE_SALT: &str =
    "INSERT INTO accounts.salts ( id, salt ) VALUES (?, ?);";

/// DatabaseConfig defines Scylla authentification
struct DatabaseConfig {
    host: String,
    username: String,
    password: String,
}

impl DatabaseConfig {
    fn new(config: crate::model::config::Config) -> Self {
        Self {
            host: config.database.scylla.hosts[0].clone(),
            username: config
                .database
                .scylla
                .username
                .unwrap_or_else(|| "cassandra".to_string()),
            password: config
                .database
                .scylla
                .password
                .unwrap_or_else(|| "cassandra".to_string()),
        }
    }
}

/// Inits scylla (or cassandra) database connection
pub async fn init(
    config: crate::model::config::Config,
) -> Result<Session, scylla::transport::errors::NewSessionError> {
    let config = DatabaseConfig::new(config);

    SessionBuilder::new()
        .known_node(config.host)
        .user(config.username, config.password)
        .build()
        .await
}

/// This function allows to create every needed tables
/// to work properly with the program
pub async fn create_tables(conn: &Session) {
    conn.query(CREATE_USERS_TABLE, &[])
        .await
        .expect("accounts.users creation error");
    conn.query(CREATE_BOTS_TABLE, &[])
        .await
        .expect("accounts.bots creation error");
    conn.query(CREATE_OAUTH_TABLE, &[])
        .await
        .expect("accounts.oauth creation error");
    conn.query(CREATE_TOKENS_TABLE, &[])
        .await
        .expect("accounts.tokens creation error");
    conn.query(CREATE_SALTS_TABLE, &[])
        .await
        .expect("accounts.slats creation error");
    conn.query(CREATE_USERS_INDEX_EMAIL, &[])
        .await
        .expect("second index (email");
    conn.query(CREATE_USERS_INDEX_EXPIRE_AT, &[])
        .await
        .expect("second index (expire_at");
    conn.query(CREATE_OAUTH_INDEX_USER_ID, &[])
        .await
        .expect("second index (user_id");
    conn.query(CREATE_TOKENS_INDEX_USER_ID, &[])
        .await
        .expect("second index (user_id");
}

/// Make a query to scylla (or cassandra)
pub async fn query(
    conn: &Arc<Session>,
    query: impl Into<scylla::query::Query>,
    params: impl scylla::frame::value::ValueList,
) -> Result<scylla::QueryResult, QueryError> {
    conn.query(query, params).await
}

/// Create a user into the database
pub async fn create_user(
    conn: &Arc<Session>,
    vanity: &String,
    email: String,
    username: String,
    password: String,
    phone: Option<String>,
    birthdate: Option<String>,
) -> Result<(), QueryError> {
    let user = crate::model::user::User {
        vanity: vanity.to_string(),
        username,
        avatar: Some("".to_string()),
        bio: Some("".to_string()),
        email: Some(email),
        password: Some(password),
        phone: Some(phone.unwrap_or_default()),
        birthdate: Some(birthdate.unwrap_or_default()),
        flags: 0,
        deleted: false,
        verified: false,
    };

    // Use parameterized query to prevent SQL injection
    conn.query(
        CREATE_USER,
        (
            user.vanity,
            user.email,
            user.username,
            user.password,
            user.phone,
            user.birthdate,
            user.avatar,
            user.bio,
        ),
    )
    .await?;

    Ok(())
}

/// Create a OAuth2 code
pub async fn create_oauth(
    conn: &Arc<Session>,
    id: String,
    vanity: String,
    bot_id: String,
    scope: Vec<String>,
) {
    let _ = conn
        .query(CREATE_OAUTH, (id, vanity, bot_id, scope, false))
        .await;
}

/// Create a new salt to split it and secure it
pub async fn create_salt(conn: &Arc<Session>, salt: String) -> String {
    let id = uuid::Uuid::new_v4().to_string();

    let _ = conn.query(CREATE_SALT, (id.to_string(), salt)).await;

    id
}
