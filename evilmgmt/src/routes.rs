//! Routes for the HTTP application
use axum::{
    Router,
    http::{StatusCode, Uri},
    routing::get,
};

pub fn app() -> Router {
    Router::new()
        .route("/", get(|| async { "Evilness Management" }))
        .fallback(fallback_handler)
}

async fn fallback_handler(uri: Uri) -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, format!("No route for {uri}"))
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, extract::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use super::*;

    #[tokio::test]
    async fn root_return_static_response_and_ok() {
        let routes = app();
        let request = Request::builder().uri("/").body(Body::empty()).unwrap();

        let response = routes.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body, "Evilness Management");
    }

    #[tokio::test]
    async fn nonexisting_url_returns_emply_response_and_not_found() {
        let routes = app();
        let request = Request::builder()
            .uri("/nonexisting")
            .body(Body::empty())
            .unwrap();

        let response = routes.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body, "No route for /nonexisting");
    }
}
