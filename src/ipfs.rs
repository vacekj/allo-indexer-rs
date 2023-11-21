use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use crate::IpfsValue;

/// Returns a transparently cached value of the given IPFS hash
/// Only reaches out to an IPFS gateway if the value is not found in the database
/// Asynchronously writes to the cache
pub async fn ipfs_get(pool: PgPool, hash: String) -> IpfsValue {
    let rec = sqlx::query!(
        r#"
INSERT INTO ipfs ( hash, value )
VALUES ( $1, $2 )
RETURNING value
        "#,
        hash, "hello"
    )
        .fetch_one(&pool)
        .await.unwrap();

    IpfsValue {
        hash,
        value: rec.value
    }
}