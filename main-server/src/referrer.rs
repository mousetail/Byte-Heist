use axum::{Extension, body::Body, extract::Request, middleware::Next, response::Response};
use tower_sessions::Session;

pub async fn referrer_layer(
    Extension(session): Extension<Session>,
    request: Request,
    next: Next,
) -> Response {
    let referrer = request.headers().get("referer");
    if let Some(referrer) = referrer {
        let Ok(current_value) = session.get::<String>("referrer").await else {
            return Response::builder()
                .header("Content-Type", "Text/Plain")
                .body(Body::from("Failure loading session"))
                .expect("Failure building response");
        };
        if current_value.is_none()
            && let Err(e) = session
                .insert("referrer", String::from_utf8_lossy(referrer.as_bytes()))
                .await
            {
                eprintln!("Error saving session {e:?}");
                return Response::builder()
                    .header("Content-Type", "Text/Plain")
                    .body(Body::from("Failure saving session"))
                    .expect("Failure building response");
            };
    }

    next.run(request).await
}
