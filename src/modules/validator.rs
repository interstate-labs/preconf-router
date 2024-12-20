use axum::{
    async_trait,
    extract::{Request, FromRequest},
    response::{Response, IntoResponse},
    body::Bytes,
    http::StatusCode
};

use crate::spec::PreconfRequestParams;
pub struct ValidatedBody(pub PreconfRequestParams);

#[async_trait]
impl<S> FromRequest<S> for ValidatedBody
where
    Bytes: FromRequest<S>,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let body = Bytes::from_request(req, state)
            .await
            .map_err(IntoResponse::into_response)?;

        let request_params = serde_json::from_slice::<PreconfRequestParams>(&body).map_err(|e| {
            tracing::error!(err = ?e, "Failed to parse preconf request");
        });
    
        match request_params {
            Ok(data) => {
                match data.signed_tx.recover_signer() {
                    Some(sender) => {
                        match data.signature.recover_address_from_prehash(&data.digest()) {
                            Ok(signer) => {
                                if **sender != **signer {
                                    return Err((StatusCode::BAD_REQUEST, "Invalid signature").into_response());
                                }
                            },
                            Err(_err) => {
                                return Err((StatusCode::BAD_REQUEST, "Failed to get signer from signature").into_response());
                            }
                        }
                    },
                    _ => {
                        return Err((StatusCode::BAD_REQUEST, "Failed to get signer from signed tx").into_response());
                    }
                }
                tracing::debug!("Received valid preconfirmation request");
                Ok(Self(data))
            }
            Err(_err) => {
                return Err((StatusCode::BAD_REQUEST, "Invalid request").into_response());
            }
        }            
    }
}