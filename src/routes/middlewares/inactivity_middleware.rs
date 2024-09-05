use std::sync::{Arc, Mutex};
use std::time::Instant;

use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use futures_util::future::LocalBoxFuture;
use std::time::Duration;

pub struct LastActivityTime(pub Mutex<Instant>);
pub struct InactivityMiddleware {
    pub last_activity: Arc<LastActivityTime>,
    pub shutdown_duration: Duration,
}

impl<S, B> Transform<S, ServiceRequest> for InactivityMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = InactivityMiddlewareService<S>;
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(InactivityMiddlewareService {
            service,
            last_activity: self.last_activity.clone(),
            shutdown_duration: self.shutdown_duration,
        }))
    }
}

pub struct InactivityMiddlewareService<S> {
    pub service: S,
    pub last_activity: Arc<LastActivityTime>,
    #[allow(dead_code)]
    pub shutdown_duration: Duration,
}

impl<S, B> Service<ServiceRequest> for InactivityMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let last_activity = self.last_activity.clone();
        let mut last_activity = last_activity.0.lock().unwrap();
        *last_activity = Instant::now();
        drop(last_activity);

        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}
