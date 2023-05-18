use newsletter::startup::run;
use secrecy::ExposeSecret;
use std::net::TcpListener;
use newsletter::configuration::get_configuration;
use sqlx::PgPool;
use newsletter::telemetry:: {init_subscriber,get_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("newsletter".into(),"info".into(), std::io::stdout);
    init_subscriber(subscriber);
    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection = PgPool::connect_lazy(&configuration.database.connection_string().expose_secret())
        .expect("Failed to connect to Postgres.");
    let address = format!("{}:{}", configuration.application.host,configuration.application.port);
    let listener = TcpListener::bind(address)?;
    run(listener, connection)?.await
}

