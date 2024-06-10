use crate::server::{
    entities::{
        friend::Column as FriendColumn,
        friend_request::Column as FriendRequestColumn,
        game::Column as GameColumn,
        prelude::{Friend, FriendRequest, Game},
    },
    extractors::User,
    handlers::StringError,
    helpers,
    state::AppState,
};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::json;
use std::sync::Arc;

/// Fetch the current user's information.
pub async fn me(user: User) -> Result<impl IntoResponse, Response> {
    Ok(user)
}

/// Fetch the games the current user is participating in.
pub async fn games(
    State(state): State<Arc<AppState>>,
    user: User,
) -> Result<impl IntoResponse, Response> {
    let games = Game::find()
        .filter(
            GameColumn::Host
                .eq(user.id.to_string())
                .or(GameColumn::Guest.eq(user.id.to_string())),
        )
        .all(state.database.as_ref())
        .await
        .map_err(|e| StringError(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;
    let mut resp = vec![];
    for g in &games {
        let id = if user.id.to_string() == g.host {
            &g.guest
        } else {
            &g.host
        };
        let opponent = helpers::get_user(&state, id, false).await?;
        resp.push(json!({
            "id": g.id,
            "opponent": opponent.username
        }));
    }
    Ok(super::Response::new(resp, StatusCode::OK))
}

/// Fetch the friend requests the current user has received.
pub async fn incoming(
    State(state): State<Arc<AppState>>,
    user: User,
) -> Result<impl IntoResponse, Response> {
    let frs = FriendRequest::find()
        .filter(FriendRequestColumn::Recipient.eq(user.id))
        .all(state.database.as_ref())
        .await
        .map_err(|e| StringError(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;
    let mut incoming = vec![];
    for fr in &frs {
        let sender = helpers::get_user(&state, &fr.sender.to_string(), false).await?;
        incoming.push(json!({
            "sender": sender.username,
        }));
    }
    Ok(super::Response::new(incoming, StatusCode::OK))
}

/// Fetch the friend requests the current user has sent.
pub async fn outgoing(
    State(state): State<Arc<AppState>>,
    user: User,
) -> Result<impl IntoResponse, Response> {
    let frs = FriendRequest::find()
        .filter(FriendRequestColumn::Sender.eq(user.id))
        .all(state.database.as_ref())
        .await
        .map_err(|e| StringError(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;
    let mut outgoing = vec![];
    for fr in &frs {
        let recipient = helpers::get_user(&state, &fr.recipient.to_string(), false).await?;
        outgoing.push(json!({
            "recipient": recipient.username,
        }));
    }
    Ok(super::Response::new(outgoing, StatusCode::OK))
}

/// Fetch the friends of the current user.
pub async fn friends(
    State(state): State<Arc<AppState>>,
    user: User,
) -> Result<impl IntoResponse, Response> {
    let friends = Friend::find()
        .filter(FriendColumn::A.eq(user.id).or(FriendColumn::B.eq(user.id)))
        .all(state.database.as_ref())
        .await
        .map_err(|e| StringError(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;
    let mut f = vec![];
    for friend in &friends {
        let id = if friend.a == user.id {
            &friend.b
        } else {
            &friend.a
        };
        let friend = helpers::get_user(&state, &id.to_string(), false).await?;
        f.push(json!({
            "username": friend.username,
        }));
    }
    Ok(super::Response::new(f, StatusCode::OK))
}

#[cfg(test)]
mod tests {
    use crate::server::{self, handlers::Response};
    use test_utils::{function, Client, Map};

    #[tokio::test]
    async fn me() {
        let database = sea_orm::Database::connect(server::INSECURE_DEFAULT_DATABASE_URL)
            .await
            .unwrap();
        let url = test_utils::init(crate::server::app(database)).await;
        let client = Client::authenticated(&[function!()], &url, true).await;
        let resp: Response<Map> = client.get(&url, "/@me").await;
        assert_eq!(&resp.message["username"], function!());
    }
}
