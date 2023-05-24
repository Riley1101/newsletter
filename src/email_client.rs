use std::{format, println};

use crate::domain::SubscriberEmail;
use reqwest::Client;
use secrecy::{Secret, ExposeSecret};

#[derive(Debug)]
pub struct EmailClient{
    http_client: Client,
    base_url: String,
    sender: SubscriberEmail,
    authorization_token: Secret<String>,
}


#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
#[derive(Debug)]
pub struct SendEmailRequest<'a>{
    from :&'a str,
    to : &'a str,
    subject: &'a str,
    html_body: &'a str,
    text: &'a str,
}

impl EmailClient {
    pub fn new(base_url: String, sender: SubscriberEmail, authorization_token:Secret<String>,timeout:std::time::Duration) -> Self {
        let http_client = Client::builder()
            .timeout(timeout)
            .build()
            .unwrap();
        Self {
            http_client,
            base_url,
            sender,
            authorization_token
        }
    }
    pub async fn send_email(
        &self,
        recipent: &SubscriberEmail,
        subject : &str,
        html_content: &str,
        text: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}", self.base_url);
        let request_body = SendEmailRequest {
            from: self.sender.as_ref(),
            to: recipent.as_ref(),
            subject,
            html_body: html_content,
            text
        };
        println!("{:?}",&request_body);
        let _ = self.http_client
            .post(&url)
            .header("AUTHORIZATION",format!("Basic {}",self.authorization_token.expose_secret()))
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}


#[cfg(test)]
mod tests{
    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;
    use claim::{assert_err,assert_ok};
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::Paragraph;
    use fake::{Fake,Faker};
    use secrecy::Secret;
    use wiremock::{Mock,MockServer,ResponseTemplate,Request};
    use wiremock::matchers::{header, header_exists, method};

    fn content()->String{
        Paragraph(1..10).fake()
    }
    fn subject()->String{
        Paragraph(1..2).fake()
    }

    fn email()->SubscriberEmail{
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }

    fn email_client (base_url:String)->EmailClient{
        EmailClient::new(base_url, email(), Secret::new(Faker.fake()), std::time::Duration::from_secs(100))
    }

    struct SendEmailBodyMatcher;
    impl wiremock::Match for SendEmailBodyMatcher {
       fn matches(&self, request: &Request) -> bool {
           let result : Result<serde_json::Value, _> = serde_json::from_slice(&request.body);
           if let Ok(body) = result {
                dbg!(&body);
                body.get("From").is_some()
                    && body.get("To").is_some()
                && body.get("Subject").is_some()
                && body.get("HtmlBody").is_some()
                && body.get("Text").is_some()
           }else {
               false
           }
       }
    }

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url(){
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());
        Mock::given(header_exists("AUTHORIZATION")) 
            .and(header("Content-Type", "application/json"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;
        let outcome = email_client.send_email(&email(), &subject(), &content(), &content()).await;
        assert!(outcome.is_ok());
    }

    #[tokio::test]
    async fn send_email_failed_if_server_returns_500(){
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());
        Mock::given(header_exists("AUTHORIZATION")) 
            .and(header("Content-Type", "application/json"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;
        let outcome = email_client.send_email(&email(), &subject(), &content(), &content()).await;
        assert_err!(outcome);
    }

    #[tokio::test]
    async fn send_email_timeout_if_server_takes_too_long(){
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());
        let response = ResponseTemplate::new(200)
            .set_delay(std::time::Duration::from_secs(10));
        Mock::given(wiremock::matchers::any()) 
            .and(header("Content-Type", "application/json"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;
        let outcome = email_client.send_email(&email(), &subject(), &content(), &content()).await;
        assert!(outcome.is_ok());
    }

}
