
use disposables::context::Context;
use disposables::container::ContainerParams;
use disposables::protocol::V1Event;
use sqlx::postgres::PgPoolOptions;

#[tokio::test]
async fn normal_server() {
    drop(env_logger::try_init());

    log::info!("Creating context...");
    let ctx = Context::new().unwrap();
    log::info!("Creating container...");
    let mut container = ContainerParams::new("docker.io/postgres:alpine")
        .env("POSTGRES_PASSWORD", "postgres")
        .port(5432)
        .wait_for_cmd(["pg_isready"], 500)
        .create(&ctx).unwrap();

    log::info!("Waiting for container to be ready...");
    assert!(matches!(container.wait(), Ok(V1Event::Ready)),
        "Container start failed, Logs: {}", container.logs().unwrap());
    log::info!("Container is now ready");

    let pool = async {
        for addr in container.port(5432).unwrap() {
            match PgPoolOptions::new()
                .connect(&format!("postgres://postgres:postgres@{addr}/postgres"))
                .await 
            { 
                Ok(x) => return x,
                Err(e) => log::info!("Connect({addr}): {e}"),
            }
        }
        panic!("Cannot connect to database");
    }.await;

    sqlx::query("CREATE TABLE test(id INTEGER);")
        .execute(&pool).await.unwrap();
}
