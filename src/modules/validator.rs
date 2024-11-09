use axum::{
  async_trait, body::{Body, Bytes}, extract::{FromRequest, Path, Request}, middleware::Next, response::{IntoResponse, Response}, Json, RequestExt
};
use reqwest::StatusCode;

use futures_util::future::BoxFuture;

use tower::{Layer, Service};
use std::{ sync::Arc, task::{Context, Poll}};

use super::proposer_router::ProposerRouter;

#[derive(Clone)]
pub struct ValidatorLayer {
    pub proposer_router: Arc<ProposerRouter>,
}
impl<S> Layer<S> for ValidatorLayer {
    type Service = ValidatorService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ValidatorService {
            inner,
            proposer_router: self.proposer_router.clone(),
        }
    }
}

#[derive(Clone)]
pub struct ValidatorService<S> {
    pub inner: S,
    pub proposer_router: Arc<ProposerRouter>,
}

impl<S> Service<Request> for ValidatorService<S>
where
S: Service<Request<Body>, Response = Response> + Clone + Send + 'static,
S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
      
      let future = self.inner.call(request);
      Box::pin(async move {
          let response: Response = future.await?;
          Ok(response)
      })
  }
  
}
