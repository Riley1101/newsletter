use std::net::TcpListener;
use newsletter::startup::run;
use sqlx::{Connection, PgConnection};
use newsletter::configuration::get_configuration;

#[tokio::test]
async fn health_check_works(){
   let address =spawn_app()
       .await;
   let client = reqwest::Client::new();
   let response = client
       .get(&format!("{}/health_check", &address))
       .send()
       .await
       .expect("failed to execute request");
   assert!(response.status().is_success());
   assert_eq!(Some(0),response.content_length());
}
#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data(){
    let address =spawn_app()
       .await;
    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &address))
        .header("Content-Type","application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");
    assert_eq!(200,response.status().as_u16());
}

#[tokio::test]
async fn subscribe_returns_400_for_valid_form_data(){
    let address = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin","missing the email"),
        ("email=ursula_le_guin%40gmail.com","missing the name"),
        ("","missing both name and email")
    ];
    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscriptions", &address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

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

async fn spawn_app() -> String{ 
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
  let configuration = get_configuration().expect("Failed to read configuration.");
    let connection = PgConnection::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    println!("port: {}",port);
    let server = run(listener,connection).expect("expected to bind address");
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{}",port)
}
