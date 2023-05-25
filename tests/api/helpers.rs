use std::net::TcpListener;
use std::{println, format};
use newsletter::email_client::EmailClient;
use once_cell::sync::Lazy;
use newsletter::{startup::run, configuration::DatabaseSettings};
use newsletter::configuration::{get_configuration, self};
use newsletter::telemetry::{get_subscriber, init_subscriber};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use wiremock::MockServer;


// make sure tracing only run once
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok(){
        let subscriber = get_subscriber("test".into(), "debug".into(),std::io::stdout);
        init_subscriber(subscriber);
    }else{
        let subscriber = get_subscriber(subscriber_name.into(),default_filter_level.into(),std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub email_server : MockServer,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body:String) -> reqwest::Response{
       reqwest::Client::new()
           .post(&format!("{}/subscriptions",self.address))
           .header("Content-Type","application/x-www-form-urlencoded")
           .body(body)
           .send()
           .await
           .expect("Failed to execute request.")
    }
}

pub async fn spawn_app() -> TestApp{ 
    Lazy::force(&TRACING);
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let connection = configue_database(&configuration.database).await;
    let sender_email = configuration.email_client.sender().expect("Invalid sender email address");
    let db_pool = connection;
    println!("port: {}",port);
    
    let timeout = configuration.email_client.timeout();
    let email_client = EmailClient::new(configuration.email_client.base_url,sender_email,configuration.email_client.authorization_token,timeout);
    let email_server = MockServer::start().await;
    let server = run(listener,db_pool.clone(),email_client).expect("expected to bind address");
    let _ = tokio::spawn(server);
    let address = format!("http://127.0.0.1:{}",port);
    let configuration = {
        let mut c = get_configuration().expect("failed to read configuration");
        c.email_client.base_url = email_server.uri();
    };
    println!("address: {:?}",configuration);
    TestApp{
        address,
        db_pool, 
        email_server
    }
}

pub async fn configue_database(config:&DatabaseSettings) ->PgPool{
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres.");
    connection
        .execute(&*format!(r#"CREATE DATABASE "{}";"#,config.database_name).as_str())
        .await
        .expect("Failed to create database.");
    let connection_pool = PgPool::connect_with(config.with_db()).await.expect("failed to connect to postgres");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate database.");
    
    connection_pool
}

