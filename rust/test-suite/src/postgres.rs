/*
 * Copyright 2024 Akash Rawal
 *
 * This file is part of Disposables.
 *
 * Disposables is free software: you can redistribute it and/or modify it under 
 * the terms of the GNU General Public License as published by the 
 * Free Software Foundation, either version 3 of the License, or 
 * (at your option) any later version.
 * 
 * Disposables is distributed in the hope that it will be useful, 
 * but WITHOUT ANY WARRANTY; without even the implied warranty of 
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. 
 * See the GNU General Public License for more details.
 * 
 * You should have received a copy of the GNU General Public License 
 * along with Disposables. If not, see <https://www.gnu.org/licenses/>. 
 */

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
        .wait_for_cmd(["pg_isready", "-h", "127.0.0.1"], 500)
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
