use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub enum HttpError {
    BadRequest(anyhow::Error),
    InternalServerError(anyhow::Error),
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        match self {
            HttpError::BadRequest(err) => {
                (StatusCode::BAD_REQUEST, format!("Bad Request: {}", err)).into_response()
            }
            HttpError::InternalServerError(err) => {
                tracing::error!("Internal Server Error: {}", err);

                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Internal Server Error"),
                )
                    .into_response()
            }
        }
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for HttpError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self::InternalServerError(err.into())
    }
}
