use std::fmt::{Debug, Formatter};
use std::str::FromStr;

use crate::db::RepoError;
use async_trait::async_trait;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::Pool;
use uuid::Uuid;

pub struct GreetingRepositoryImpl {
    pool: Box<Pool<sqlx::Postgres>>,
}

#[async_trait]
pub trait GreetingRepository {
    async fn store(&mut self, greeting: Greeting) -> Result<(), RepoError>;
}

impl Debug for GreetingRepositoryImpl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "GreetingRepository")
    }
}
impl GreetingRepositoryImpl {
    pub async fn new(pool: Box<Pool<sqlx::Postgres>>) -> Result<Self, RepoError> {
        Ok(Self { pool })
    }
}
#[async_trait]
impl GreetingRepository for GreetingRepositoryImpl {
    async fn store(&mut self, greeting: Greeting) -> Result<(), RepoError> {
        let mut transaction = self.pool.begin().await?;

        let id: (i64,) = sqlx::query_as("INSERT INTO greeting(message_id, \"from\", \"to\", heading, message, created) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id")
            .bind(Uuid::from_str(&*greeting.id).unwrap())
            .bind(greeting.from)
            .bind(greeting.to)
            .bind(greeting.heading)
            .bind(greeting.message)
            .bind(greeting.created)
            .fetch_one(&mut *transaction).await?;

        sqlx::query("INSERT INTO ikke_paa_logg(greeting_id) VALUES ($1)")
            .bind(id.0)
            .execute(&mut *transaction)
            .await?;

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
