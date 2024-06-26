use crate::server::{
    handlers::{Response, StringError},
    helpers,
    state::AppState,
    strings,
};
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts, State},
    http::{request::Parts, StatusCode},
    response::IntoResponse,
};
use axum_extra::extract::CookieJar;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

pub struct User {
    pub id: Uuid,
    pub username: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for User
where
    S: Send + Sync,
    Arc<AppState>: FromRef<S>,
{
    type Rejection = StringError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Extract the session cookie from the app state. We use `from_request_parts` because we want to be able
        // to use other extractors after this one, and `from_request` consumes the request.
        let jar = CookieJar::from_request_parts(parts, state).await.unwrap();
        let state: State<Arc<AppState>> = State::from_request_parts(parts, state).await.unwrap();
        let sid = jar
            .get(strings::SESSION_COOKIE_NAME)
            .ok_or(StringError(
                strings::INVALID_TOKEN.into(),
                StatusCode::UNAUTHORIZED,
            ))?
            .value_trimmed();
        // Fetch the session associated with the cookie and then fetch the user associated with the session.
        let session = helpers::get_session(&state, sid).await?;
        let user = helpers::get_user(&state, &session, false).await?;
        Ok(User {
            id: user.id,
            username: user.username,
        })
    }
}

impl IntoResponse for User {
    fn into_response(self) -> axum::response::Response {
        Response::new(
            json!({
                "id": self.id,
                "username": self.username,
            }),
            StatusCode::OK,
        )
        .into_response()
    }
}
