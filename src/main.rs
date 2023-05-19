use newsletter::startup::run;
use std::net::TcpListener;
use newsletter::configuration::get_configuration;
use newsletter::telemetry:: {init_subscriber,get_subscriber};
use newsletter::email_client::EmailClient;
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("newsletter".into(),"info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let sender_email = configuration.email_client.sender().expect("Invalid sender email address");
    let email_client = EmailClient::new(configuration.email_client.base_url,sender_email,configuration.email_client.authorization_token);

    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());
    let address = format!("{}:{}", configuration.application.host,configuration.application.port);
    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool,email_client)?.await
}

