use std::net::TcpListener;
use newsletter::{startup::run, configuration::DatabaseSettings};
use newsletter::configuration::get_configuration;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

#[tokio::test]
async fn health_check_works(){
   let app =spawn_app()
       .await;
   let client = reqwest::Client::new();
   let response = client
       .get(&format!("{}/health_check", &app.address))
       .send()
       .await
       .expect("failed to execute request");
   assert!(response.status().is_success());
   assert_eq!(Some(0),response.content_length());
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data(){
    let app =spawn_app()
       .await;
    let client = reqwest::Client::new();
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type","application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");
    let _ = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(200,response.status().as_u16());
}

#[tokio::test]
async fn subscribe_returns_400_for_valid_form_data(){
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin","missing the email"),
        ("email=ursula_le_guin%40gmail.com","missing the name"),
        ("","missing both name and email")
    ];
    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");
        let _ = sqlx::query!("SELECT email, name FROM subscriptions",)
            .fetch(&app.db_pool);
        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            // Additional customised error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

async fn spawn_app() -> TestApp{ 
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let connection = configue_database(&configuration.database).await;
    let db_pool = connection;
    println!("port: {}",port);
    let server = run(listener,db_pool.clone()).expect("expected to bind address");
    let _ = tokio::spawn(server);
    let address = format!("http://127.0.0.1:{}",port);
    TestApp{
        address,
        db_pool, 
    }
}

pub(crate) async fn configue_database(config:&DatabaseSettings) ->PgPool{
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres.");
    connection
        .execute(&*format!(r#"CREATE DATABASE "{}";"#,config.database_name).as_str())
        .await
        .expect("Failed to create database.");
    let connection_pool = PgPool::connect(&config.connection_string()).await;
    match connection_pool {
        Ok(pool) => {
            sqlx::migrate!("./migrations")
                .run(&pool)
                .await
                .expect("Failed to migrate database.");
            pool
        },
        Err(_) => panic!("migration error"),
    }
}
