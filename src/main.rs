use axum::{
    extract::{Path, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use thiserror::Error;
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiError {
    pub message: String,
    pub status_code: u16,
}

#[derive(Error, Debug)]
enum AppError {
    #[error("Database Error: {0}")]
    DatabaseError(sqlx::Error),
    #[error("URL Parse Error")]
    URLParseError,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let code = match self {
            AppError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::URLParseError => StatusCode::UNPROCESSABLE_ENTITY,
        };
        (
            code,
            [(
                header::CONTENT_TYPE,
                HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
            )],
            Json(ApiError {
                status_code: code.as_u16(),
                message: self.to_string(),
            }),
        )
            .into_response()
    }
}

#[shuttle_runtime::main]
async fn axum(#[shuttle_shared_db::Postgres] pool: PgPool) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to migrate database");

    let router = Router::new()
        .route("/", post(shorten))
        .route("/:id", get(redirect))
        .with_state(pool);

    Ok(router.into())
}

#[derive(Serialize, FromRow)]
struct Urls {
    id: String,
    url: String,
}

async fn redirect(
    State(pool): State<PgPool>,
    Path(id): Path<String>,
) -> Result<Redirect, AppError> {
    let url = sqlx::query_as::<_, Urls>("SELECT * FROM urls WHERE id = $1")
        .bind(&id)
        .fetch_one(&pool)
        .await
        .map(|u| u.url)
        .map_err(|err| AppError::DatabaseError(err))?;
    Ok(Redirect::to(&url))
}

#[derive(serde::Deserialize)]
pub struct Input {
    url: String,
}

async fn shorten(State(pool): State<PgPool>, input: Json<Input>) -> Result<String, AppError> {
    let id = nanoid::nanoid!(6);

    let parsed_url = Url::parse(&input.url).map_err(|_| AppError::URLParseError)?;

    sqlx::query("INSERT INTO urls (id, url) VALUES ($1, $2)")
        .bind(&id)
        .bind(parsed_url.as_str())
        .execute(&pool)
        .await
        .map_err(|err| AppError::DatabaseError(err))?;

    Ok(format!("https://mijikaku.shuttleapp.rs/{id}"))
}
