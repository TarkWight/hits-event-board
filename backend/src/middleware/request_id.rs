use std::task::{Context, Poll};
use tower::{Layer, Service};
use http::{Request, HeaderValue};
use uuid::Uuid;

#[derive(Clone, Default)]
pub struct RequestIdLayer;
impl RequestIdLayer { pub fn new() -> Self { Self } }

#[derive(Clone)]
pub struct RequestIdMiddleware<S> { inner: S }

impl<S> Layer<S> for RequestIdLayer {
    type Service = RequestIdMiddleware<S>;
    fn layer(&self, inner: S) -> Self::Service { RequestIdMiddleware { inner } }
}

impl<S, B> Service<Request<B>> for RequestIdMiddleware<S>
where S: Service<Request<B>> {
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> { self.inner.poll_ready(cx) }
    fn call(&mut self, mut req: Request<B>) -> Self::Future {
        let rid = Uuid::new_v4().to_string();
        req.headers_mut().insert("x-request-id", HeaderValue::from_str(&rid).unwrap());
        self.inner.call(req)
    }
}
