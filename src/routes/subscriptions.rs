use actix_web::{web, HttpResponse};
use serde;
use sqlx::PgPool;
use chrono::Utc;
use uuid::Uuid;
use tracing;
use crate::domain::{SubscriberName, NewSubscriber};

#[derive(serde::Deserialize)]
#[derive(Debug)]
pub struct FormData {
    email: String,
    name: String,
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, pool)
)]
pub async fn insert_subscriber(pool:&PgPool, new_subscriber: &NewSubscriber) -> Result<(), sqlx::Error>{
    let request_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        request_id,
        new_subscriber.email,
        new_subscriber.name.as_ref(),
        Utc::now())
        .execute(pool)
        .await
        .map_err(|e|{
            tracing::error!("Failed to execute query: {:?}", e);
            e
        })?;
    Ok(())
}

pub async fn subscribe(form:web::Form<FormData>,pool:web::Data<PgPool>) ->HttpResponse{
    let new_subscriber = NewSubscriber{
        name : SubscriberName::parse(form.0.name).expect("expect to be name"),
        email : form.0.email,
    };
    match insert_subscriber(&pool, &new_subscriber).await {
       Ok(_) => HttpResponse::Ok().finish(),
       Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

