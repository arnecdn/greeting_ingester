use std::str::FromStr;
use async_trait::async_trait;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{migrate,  Pool};
use sqlx::migrate::MigrateError;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;


pub struct GreetingRepositoryImpl{
    pool: Pool<sqlx::Postgres>,
}

pub struct RepoError{
    pub error_message: String,
}

impl From<sqlx::Error> for RepoError{
    fn from(value: sqlx::Error) -> Self {
        RepoError{ error_message: value.to_string()}
    }
}
impl From<MigrateError> for RepoError{
    fn from(value: MigrateError) -> Self {
        RepoError{ error_message: value.to_string()}
    }
}


#[async_trait]
pub trait GreetingRepository{
    async fn store(&mut self, greeting: Greeting) -> Result<(), RepoError>;
}

impl GreetingRepositoryImpl{
    pub async fn new(db_url: String)-> Result<Self, RepoError>{

        let pool = PgPoolOptions::new()
            .max_connections(100)
            .connect(&*db_url).await?;
        migrate!("./migrations")
            .run(&pool).await?;

        Ok(Self{pool })
    }
}
#[async_trait]
impl GreetingRepository for GreetingRepositoryImpl {
     async fn store (&mut self, greeting: Greeting) -> Result<(), RepoError> {
        let mut transaction = self.pool.begin().await?;
        sqlx::query!("INSERT INTO greeting(message_id, \"from\", \"to\", heading, message, created) VALUES ($1, $2, $3, $4, $5, $6)",
            Uuid::from_str(&*greeting.id).unwrap(), greeting.from,greeting.to, greeting.heading, greeting.message, greeting.created)
            .execute(&mut *transaction).await?;
        transaction.commit().await?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Greeting {
    id: String,
    to: String,
    from: String,
    heading: String,
    message: String,
    created: NaiveDateTime,
}




