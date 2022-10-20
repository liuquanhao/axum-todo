use std::{env, sync::Arc};
use tokio_postgres::{connect, Client, NoTls};

pub struct PgDatabase {
    pub client: Arc<Client>,
}

impl PgDatabase {
    pub async fn new(pg_url: &str) -> Self {
        let (client, conn) = connect(pg_url, NoTls).await.expect("无法连接数据库");
        tokio::spawn(async move {
            if let Err(error) = conn.await {
                eprintln!("数据库连接失败：{}", error);
            }
        });
        PgDatabase {
            client: Arc::new(client),
        }
    }

    pub fn get_pg_url(env_key: &str) -> String {
        match env::var(env_key) {
            Ok(url) => url,
            Err(_) => panic!("需要设置环境变量POSTGRESQL_URL为postgresql连接"),
        }
    }
}
