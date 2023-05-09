use newsletter::run;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080");
    match listener {
        Ok(listener) => {
            run(listener)?.await
        },
        Err(_) => todo!(),
    }
}
