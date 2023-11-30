// use crate::IpfsValue;
// use eyre::Result;
// use serde::{Deserialize, Serialize};
// use sqlx::PgPool;
//
// /// Returns a transparently cached value of the given IPFS hash
// /// Only reaches out to an IPFS gateway if the value is not found in the database
// /// Asynchronously writes to the cache
// pub async fn ipfs_get(pool: PgPool, hash: String) -> Result<IpfsValue> {
//     /* Check if value is in cache*/
//     let result = sqlx::query!(
//         r#"
// SELECT value
// FROM ipfs
// WHERE hash = $1
//         "#,
//         hash
//     )
//     .fetch_one(&pool)
//     .await;
//
//     if let Ok(value) = result {
//         println!("found in cache");
//         return Ok(IpfsValue {
//             hash,
//             value: value.value,
//         });
//     }
//
//     /* If value is not available, fetch it and insert it */
//     let value = reqwest::get(format!("https://cloudflare-ipfs.com/ipfs/{}", hash))
//         .await?
//         .text()
//         .await?;
//
//     sqlx::query!(
//         r#"
// INSERT INTO ipfs ( hash, value )
// VALUES ( $1, $2 )
//         "#,
//         hash,
//         value
//     )
//     .execute(&pool)
//     .await?;
//
//     Ok(IpfsValue { hash, value })
// }
