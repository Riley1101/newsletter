use actix_web::{web, HttpResponse};
use serde;

#[derive(serde::Deserialize)]
#[derive(Debug)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(_form:web::Form<FormData>) ->HttpResponse{
    println!("{:?}",_form);
    HttpResponse::Ok().finish()
}
