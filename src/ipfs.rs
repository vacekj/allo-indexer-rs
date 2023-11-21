use sqlx::PgPool;

/// Returns a transparently cached value of the given IPFS hash
/// Only reaches out to an IPFS gateway if the value is not found in the database
/// Asynchronously writes to the cache
pub async fn ipfs_get(pool: PgPool, hash: String) {
    let rec = sqlx::query!(
        r#"
INSERT INTO ipfs ( hash, value )
VALUES ( $1, $2 )
RETURNING hash
        "#,
        hash, "hello"
    )
        .fetch_one(pool)
        .await;
}