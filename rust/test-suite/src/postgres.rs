
use disposables::async_util::try_use;
use disposables::container::ContainerParams;
use disposables::protocol::V1Event;
use sqlx::postgres::PgPoolOptions;

#[tokio::test]
async fn normal_server() {
    drop(env_logger::try_init());

    let mut container = ContainerParams::new("docker.io/postgres:alpine")
        .env("POSTGRES_PASSWORD", "postgres")
        .port(5432)
        .wait_for_cmd(["pg_isready"], 500)
        .create().unwrap();

    assert!(matches!(container.wait(), Ok(V1Event::Ready)),
        "Container start failed, Logs: {}", container.logs().unwrap());

    let pool = try_use(container.port(5432).unwrap(), |addr| {
        let addr = format!("postgres://postgres:postgres@{addr}/postgres");
        async move {
            PgPoolOptions::new().connect(&addr).await
        }
    }).await.unwrap();

    sqlx::query("CREATE TABLE test(id INTEGER);")
        .execute(&pool).await.unwrap();
}
