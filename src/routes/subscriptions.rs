use std::println;

use actix_web::{web, HttpResponse};
use serde;
use sqlx::PgPool;
use chrono::Utc;
use uuid::Uuid;
use tracing;
use crate::domain::{SubscriberName,NewSubscriber, SubscriberEmail};
use crate::email_client::EmailClient; 

#[derive(serde::Deserialize)]
#[derive(Debug)]
pub struct FormData {
    email: String,
    name: String,
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip( form,pool,email_client)
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(form:web::Form<FormData>,pool:web::Data<PgPool>, email_client:web::Data<EmailClient>) ->HttpResponse{
    let name = match SubscriberName::parse(form.0.name){
        Ok(name) => name,
        Err(err) => return HttpResponse::BadRequest().body(err.to_string()),
    };
    let email = match SubscriberEmail::parse(form.0.email) {
        Ok(email) => email,
        Err(err) => return HttpResponse::BadRequest().body(err.to_string()),
    };
    let new_subscriber = NewSubscriber { email, name } ;

    match insert_subscriber(&pool, &new_subscriber).await {
       Ok(_) => HttpResponse::Ok().finish(),
       Err(err) => {
            HttpResponse::Ok().finish()
       },
    };
    if email_client.send_email(&new_subscriber.email, "welcom", "welcoem", "welcome").await.is_err(){
        println!("error is atsend email");
    }
    HttpResponse::Ok().finish()
}

pub async fn insert_subscriber(pool:&PgPool, new_subscriber: &NewSubscriber,) -> Result<(), sqlx::Error>{
    let request_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'confirmed')
        "#,
        request_id,
        new_subscriber.email.as_ref(),
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

impl TryFrom<FormData> for NewSubscriber{
    type Error = String;
    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(Self{name, email})
    }
}


