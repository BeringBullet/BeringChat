use axum::{async_trait, extract::FromRequestParts, http::{request::Parts, HeaderMap}};

use crate::{api::AppState, domain::User, error::AppError};

pub mod sessions;
pub use sessions::Sessions;

pub struct AdminGuard;

#[async_trait]
impl FromRequestParts<AppState> for AdminGuard {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let token = extract_token(&parts.headers).ok_or(AppError::Unauthorized)?;
        
        // Check session token first
        if state.sessions.validate(&token) {
            return Ok(AdminGuard);
        }
        
        // Fall back to static admin token
        if token == state.config.admin_token {
            return Ok(AdminGuard);
        }
        
        Err(AppError::Unauthorized)
    }
}

pub struct UserGuard(pub User);

#[async_trait]
impl FromRequestParts<AppState> for UserGuard {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let token = extract_token(&parts.headers).ok_or(AppError::Unauthorized)?;

        // Check user session first
        if let Some(user_id) = state.sessions.validate_user_session(&token) {
            if let Some(user) = state.store.get_user_by_id(user_id)? {
                return Ok(UserGuard(user));
            }
        }

        // Fall back to permanent DB token for backwards compat
        let user = state
            .store
            .get_user_by_token(&token)?
            .ok_or(AppError::Unauthorized)?;
        Ok(UserGuard(user))
    }
}

fn extract_token(headers: &HeaderMap) -> Option<String> {
    if let Some(value) = headers.get("x-admin-token") {
        return value.to_str().ok().map(|s| s.to_string());
    }
    if let Some(value) = headers.get("authorization") {
        let header = value.to_str().ok()?;
        let header = header.trim();
        if let Some(token) = header.strip_prefix("Bearer ") {
            return Some(token.to_string());
        }
    }
    None
}
