use axum::response::{IntoResponse, Redirect, Response};

/// POST /api/auth/logout — clear the session cookie
pub async fn logout() -> Response {
    let cookie = "concord_session=; HttpOnly; Path=/; Max-Age=0; SameSite=Lax";
    (
        [(axum::http::header::SET_COOKIE, cookie.to_string())],
        Redirect::temporary("/"),
    )
        .into_response()
}
