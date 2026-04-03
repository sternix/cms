use axum::{
    extract::Request,
    http::{Method, StatusCode},
    response::Response,
    body::Body,
};
use std::task::{Context, Poll};
use tower::{Layer, Service};
use std::future::Future;
use std::pin::Pin;

#[derive(Clone)]
pub struct CsrfLayer;

impl<S> Layer<S> for CsrfLayer {
    type Service = CsrfMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CsrfMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct CsrfMiddleware<S> {
    inner: S,
}

impl<S> Service<Request<Body>> for CsrfMiddleware<S>
where
    S: Service<Request<Body>, Response = Response> + Send + 'static + Clone,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Only check CSRF for state-changing methods on non-API-auth routes
            let method = req.method().clone();
            let path = req.uri().path().to_string();

            let needs_csrf = matches!(method, Method::POST | Method::PUT | Method::DELETE)
                && !path.starts_with("/api/auth/")
                && !path.starts_with("/api/analytics/track")
                && !path.starts_with("/api/csrf-token")
                && !path.starts_with("/api/captcha");

            if needs_csrf {
                let has_token = req.headers()
                    .get("X-CSRF-Token")
                    .and_then(|v| v.to_str().ok())
                    .is_some_and(|v| !v.is_empty());

                if !has_token {
                    let response = Response::builder()
                        .status(StatusCode::FORBIDDEN)
                        .body(Body::from("{\"error\":\"CSRF token missing\"}"))
                        .unwrap();
                    return Ok(response);
                }
            }

            inner.call(req).await
        })
    }
}
