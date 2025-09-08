use tower::{Layer, Service};
use http::Request;
use std::task::{Context, Poll};

#[derive(Clone, Default)]
pub struct JsonErrorLayer;
impl JsonErrorLayer { pub fn new() -> Self { Self } }

#[derive(Clone)]
pub struct JsonErrorMiddleware<S> { inner: S }

impl<S> Layer<S> for JsonErrorLayer {
    type Service = JsonErrorMiddleware<S>;
    fn layer(&self, inner: S) -> Self::Service { JsonErrorMiddleware { inner } }
}

impl<S, B> Service<Request<B>> for JsonErrorMiddleware<S>
where S: Service<Request<B>> {
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> { self.inner.poll_ready(cx) }
    fn call(&mut self, req: Request<B>) -> Self::Future { self.inner.call(req) }
}
