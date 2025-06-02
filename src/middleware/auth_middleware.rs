use actix_web::{
    Error, HttpMessage, HttpResponse, Result,
    body::BoxBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use futures_util::future::{LocalBoxFuture, Ready, ready};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub email: String,
    pub role: String,
    pub exp: usize,
}

pub struct AuthMiddleware {
    pub secret_key: String,
}

impl AuthMiddleware {
    pub fn new(secret_key: String) -> Self {
        Self { secret_key }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static + From<BoxBody>,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service: Rc::new(service),
            secret_key: self.secret_key.clone(),
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
    secret_key: String,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static + From<BoxBody>,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
        let secret_key = self.secret_key.clone();

        Box::pin(async move {
            // Extract Authorization header
            let auth_header = req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok());

            if let Some(auth_str) = auth_header {
                if let Some(token) = auth_str.strip_prefix("Bearer ") {
                    // Decode JWT token
                    let decoding_key = DecodingKey::from_secret(secret_key.as_ref());
                    let validation = Validation::new(Algorithm::HS256);

                    match decode::<Claims>(token, &decoding_key, &validation) {
                        Ok(token_data) => {
                            // Parse user_id from claims
                            if let Ok(user_id) = Uuid::parse_str(&token_data.claims.sub) {
                                // Store user info in request extensions
                                req.extensions_mut().insert(AuthenticatedUser {
                                    user_id,
                                    email: token_data.claims.email,
                                    role: token_data.claims.role,
                                });

                                // Continue with the request
                                return service.call(req).await;
                            }
                        }
                        Err(e) => {
                            log::error!("JWT decode error: {}", e);
                        }
                    }
                }
            }

            // Return unauthorized response
            let (req, _payload) = req.into_parts();
            let response = HttpResponse::Unauthorized()
                .json("Invalid or missing authorization token")
                .map_into_boxed_body();

            // Convert the BoxBody to B using the From trait
            let response = response.map_body(|_, body| B::from(body));

            Ok(ServiceResponse::new(req, response))
        })
    }
}

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub email: String,
    pub role: String,
}

pub fn get_authenticated_user(req: &ServiceRequest) -> Option<AuthenticatedUser> {
    req.extensions().get::<AuthenticatedUser>().cloned()
}

// Helper function for handlers to get current user
use actix_web::HttpRequest;

pub fn get_current_user(req: &HttpRequest) -> Result<AuthenticatedUser, HttpResponse> {
    req.extensions()
        .get::<AuthenticatedUser>()
        .cloned()
        .ok_or_else(|| HttpResponse::Unauthorized().json("User not authenticated"))
}
