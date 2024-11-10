use std::sync::RwLock;
use std::time::Duration;
use actix_web::{get, HttpResponse, ResponseError};
use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use derive_more::{Display};
use sqlx::{Executor, Pool};
use crate::db::RepoError;
use crate::greeting_log::ApiError::{ApplicationError};
// use crate::greetings::RepoError;

#[get("/logs")]
async fn list_log_entries(data: Data< Pool<sqlx::Postgres>>)-> Result<HttpResponse, ApiError>  {
    let r = data.execute("\
        SELECT id, greeting_id FROM LOGG
    ").await?;

    Ok(HttpResponse::Ok().json(""))
}


#[derive(Debug, Display)]
pub enum ApiError {
        ApplicationError(sqlx::Error)
}


impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match *self {
            // BadClientData(_) => StatusCode::BAD_REQUEST,
            ApplicationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::INTERNAL_SERVER_ERROR
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }
}

impl From<sqlx::Error> for ApiError{
    fn from(value: sqlx::Error) -> Self {
        ApplicationError(value)
    }
}

pub async fn generate_logg(pool : Box<Pool<sqlx::Postgres>>) {
    loop {
        tokio::time::sleep(Duration::from_secs(5)).await;
        let mut transaction = pool.begin().await.expect("");
        sqlx::query("do
                        $$
                            begin
                                perform public.generate_logg();
                            end
                        $$;")

            .execute(&mut *transaction).await.expect("");

        transaction.commit().await.expect("");
    }
}
