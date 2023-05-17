use actix_web::{web, HttpResponse};
use serde;
use sqlx::PgPool;
use chrono::Utc;
use uuid::Uuid;
use tracing;

#[derive(serde::Deserialize)]
#[derive(Debug)]
pub struct FormData {
    email: String,
    name: String,
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(form, pool)
)]
pub async fn insert_subscriber(pool:&PgPool, form : &FormData) -> Result<(), sqlx::Error>{
    let request_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        request_id,
        form.email,
        form.name,
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
    match insert_subscriber(&pool, &form).await {
       Ok(_) => HttpResponse::Ok().finish(),
       Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
