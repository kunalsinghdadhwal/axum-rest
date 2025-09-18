use anyhow::Result;
use sqlx::{PgPool, database};

use tracing::info;

pub struct Db {
    pool: PgPool,
}

impl Db {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url).await?;
        info!("Connected to the database at {}", database_url);

        Self::init_db(&pool).await?;
        Ok(Self { pool })
    }

    async fn init_db(pool: &PgPool) -> Result<()> {
        sqlx::query(
            r#"
                CREATE TABLE IF NOT EXISTS users (
                id UUID PRIMARY KEY,
                name TEXT NOT NULL,
                email TEXT UNIQUE NOT NULL,
                password TEXT NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(pool)
        .await?;

        sqlx::query(
            r#"
                CREATE TABLE IF NOT EXISTS posts (
                id UUID PRIMARY KEY,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                author_id UUID NOT NULL REFERENCES users(id),
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(pool)
        .await?;

        info!("Database initialized");
        Ok(())
    }

    pub fn get_pool(&self) -> &PgPool {
        &self.pool
    }
}

pub async fn get_pg_client() -> Result<Db> {
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in environment variables");
    let db = Db::new(&database_url).await?;
    Ok(db)
}
