#[tokio::test]
async fn health_check_works(){
   spawn_app();
}

fn spawn_app(){ 
    let server = newsletter::run().expect("expected to bind address");
    let _ = tokio::spawn(server);
}
