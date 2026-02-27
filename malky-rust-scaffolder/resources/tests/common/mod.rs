use sqlx::{PgPool, postgres::PgPoolOptions, Executor};
use uuid::Uuid;
use axum_test::TestServer;

pub struct TestApp {
    pub server: TestServer,
    pub db: PgPool,
    db_name: String,
    admin_pool: PgPool,
}

impl TestApp {
    pub async fn spawn() -> Self {
        let admin_url = std::env::var("DATABASE_TEST_ADMIN_URL")
            .unwrap_or_else(|_| "postgres://user:password@localhost:5432/postgres".to_string());

        let admin_pool = PgPoolOptions::new()
            .max_connections(2)
            .connect(&admin_url)
            .await
            .expect("Failed to connect to test admin DB");

        let db_name = format!("test_{}", Uuid::new_v4().simple());

        admin_pool
            .execute(format!("CREATE DATABASE \"{}\"", db_name).as_str())
            .await
            .expect("Failed to create test database");

        let db_url = {
            let base = admin_url.trim_end_matches('/');
            let parts: Vec<&str> = base.rsplitn(2, '/').collect();
            if parts.len() == 2 {
                format!("{}/{}", parts[1], db_name)
            } else {
                format!("{}/{}", base, db_name)
            }
        };

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await
            .expect("Failed to connect to test database");

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations on test database");

        let state = __APP_NAME__::state::AppState::new(pool.clone());
        let app = __APP_NAME__::router::create_router(state);

        let server = TestServer::new(app).expect("Failed to create test server");

        Self { server, db: pool, db_name, admin_pool }
    }

    pub async fn cleanup(self) {
        drop(self.db);
        self.admin_pool
            .execute(
                format!("DROP DATABASE IF EXISTS \"{}\" WITH (FORCE)", self.db_name).as_str(),
            )
            .await
            .expect("Failed to drop test database");
    }
}
