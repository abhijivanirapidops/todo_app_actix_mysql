use crate::middleware::auth_middleware::AuthenticatedUser;
use actix_web::{
    Error, HttpMessage, HttpResponse, Result,
    body::BoxBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use futures_util::future::{LocalBoxFuture, Ready, ready};
use std::rc::Rc;

// Authorization Middleware - checks if user has required roles
pub struct AuthorizationMiddleware {
    pub allowed_roles: Vec<String>,
}

impl AuthorizationMiddleware {
    pub fn new(allowed_roles: Vec<String>) -> Self {
        Self { allowed_roles }
    }

    // Convenience method to create from string literals
    pub fn with_roles(roles: &[&str]) -> Self {
        Self {
            allowed_roles: roles.iter().map(|&s| s.to_string()).collect(),
        }
    }

    // Predefined common role combinations
    pub fn admin_only() -> Self {
        Self::with_roles(&["admin"])
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthorizationMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static + From<BoxBody>,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthorizationMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthorizationMiddlewareService {
            service: Rc::new(service),
            allowed_roles: self.allowed_roles.clone(),
        }))
    }
}

pub struct AuthorizationMiddlewareService<S> {
    service: Rc<S>,
    allowed_roles: Vec<String>,
}

impl<S, B> Service<ServiceRequest> for AuthorizationMiddlewareService<S>
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
        let allowed_roles = self.allowed_roles.clone();

        Box::pin(async move {
            // Get authenticated user from request extensions (set by AuthMiddleware)
            let user = req.extensions().get::<AuthenticatedUser>().cloned();

            match user {
                Some(authenticated_user) => {
                    // Check if user's role is in the allowed roles list
                    if allowed_roles.contains(&authenticated_user.role) {
                        // User has required role, continue with request
                        service.call(req).await
                    } else {
                        // User doesn't have required role, return forbidden
                        let (req, _payload) = req.into_parts();
                        let response = HttpResponse::Forbidden()
                            .json(serde_json::json!({
                                "error": "Access denied",
                                "message": format!(
                                    "Insufficient permissions. Required roles: {:?}, Your role: '{}'",
                                    allowed_roles, authenticated_user.role
                                ),
                                "required_roles": allowed_roles,
                                "user_role": authenticated_user.role,
                                "user_id": authenticated_user.user_id
                            }))
                            .map_into_boxed_body();

                        let response = response.map_body(|_, body| B::from(body));
                        Ok(ServiceResponse::new(req, response))
                    }
                }
                None => {
                    // User not authenticated (AuthMiddleware should have caught this)
                    // This is a fallback in case AuthMiddleware wasn't applied
                    let (req, _payload) = req.into_parts();
                    let response = HttpResponse::Unauthorized()
                        .json(serde_json::json!({
                            "error": "Authentication required",
                            "message": "You must be authenticated to access this resource"
                        }))
                        .map_into_boxed_body();

                    let response = response.map_body(|_, body| B::from(body));
                    Ok(ServiceResponse::new(req, response))
                }
            }
        })
    }
}

// Macro to easily create authorization middleware for specific roles
#[macro_export]
macro_rules! require_roles {
    ($($role:expr),+) => {
        $crate::authorization_middleware::AuthorizationMiddleware::with_roles(&[$($role),+])
    };
}
