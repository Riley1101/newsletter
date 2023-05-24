use crate::helpers::spawn_app;
use wiremock::matchers::{path,method};
use wiremock::{Mock,ResponseTemplate};

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data(){
    let app =spawn_app()
       .await;
    let client = reqwest::Client::new();
    let body = "name=%20guin&email=ursula_le_guin%40gmail.com";
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
async fn subscribe_returns_400_when_fields_present_but_invalid(){
    let app =spawn_app()
        .await;
    let client = reqwest::Client::new();
    let test_cases = vec![
      ("name=Ursula&email=", "empty email"),
      ("name=Ursula&email=definitely-not-an-email", "invalid email"),
   ];

    for (body, description) in test_cases{
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type","application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request");
        assert_eq!(
            400,
            response.status().as_u16(),
            "api did not return 400 when the payload was {}",description
        )
    }

}

#[tokio::test]
async fn subscribe_returns_400_for_valid_form_data(){
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin","missing the email"),
        ("email=ursula_le_guin%40gmail.com","missing the name"),
        ("email=something","missing both name and email")
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

#[tokio::test]
async fn subscribe_send_a_confirmation_email_for_valid_data(){
    let app = spawn_app().await;
    let body = "name=leguin&email=ursula_le_guin%40gmail.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;
    app.post_subscriptions(body.into())
        .await
        .expect("Failed to post a new subscriber");        
}


