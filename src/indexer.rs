use std::pin::Pin;

use crate::{
    event_handling::{
        event_to_changeset, events_to_change_sets_sequential, ChangeSet, Event, EventPayload,
        IpfsGetter, MetaPtr, DB_SCHEMA,
    },
    event_log::{EventLog, InMemoryEventLog},
};
use async_stream::stream;
use futures::pin_mut;
use futures::stream::Stream;
use futures::stream::StreamExt;
use sea_query::{Expr, Iden, PostgresQueryBuilder, Query};
use tokio_postgres::{Client, Connection, Error, NoTls, Transaction};

fn event_log_to_change_sets(
    event_log: &dyn EventLog,
    ipfs_getter: IpfsGetter,
) -> impl Stream<Item = ChangeSet> + '_ {
    let chunk_size = 2;
    let mut current_chunk_start_index = 0;

    stream! {
        loop {
            let (events, next_events_chunk_start_index) = event_log.range(
                current_chunk_start_index,
                current_chunk_start_index + chunk_size,
            );

            if events.len() == 0 {
                break;
            }

            let change_sets = events_to_change_sets_sequential(events, ipfs_getter).await;

            for change_set in change_sets {
                yield change_set;
            }

            current_chunk_start_index = next_events_chunk_start_index
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_log::InMemoryEventLog;

    #[tokio::test]
    async fn test_project_created() {
        let ipfs_getter = |_cid: &str| -> String { "".to_string() };
        let mut event_log = InMemoryEventLog::new();
        event_log.append(Event {
            chain_id: 1,
            address: "0x123".to_string(),
            block_number: 4242,
            payload: EventPayload::ProjectCreated {
                project_id: "proj-123".to_string(),
            },
        });

        let db_dump = event_log_to_db_dump(&event_log, ipfs_getter).await.unwrap();

        insta::assert_yaml_snapshot!(db_dump);
    }

    #[tokio::test]
    async fn test_project_created_and_metadata_updated() {
        let ipfs_getter = |_cid: &str| -> String { "{ \"foo\": \"bar\" }".to_string() };
        let mut event_log = InMemoryEventLog::new();
        event_log.append(Event {
            chain_id: 1,
            address: "0x123".to_string(),
            block_number: 4242,
            payload: EventPayload::ProjectCreated {
                project_id: "proj-123".to_string(),
            },
        });
        event_log.append(Event {
            chain_id: 1,
            address: "0x123".to_string(),
            block_number: 4242,
            payload: EventPayload::MetadataUpdated {
                project_id: "proj-123".to_string(),
                meta_ptr: MetaPtr {
                    pointer: "123".to_string(),
                },
            },
        });

        let db_dump = event_log_to_db_dump(&event_log, ipfs_getter).await.unwrap();

        insta::assert_yaml_snapshot!(db_dump);
    }

    fn dummy_ipfs_getter(_cid: &str) -> String {
        "".into()
    }

    async fn event_log_to_db_dump(
        event_log: &dyn EventLog,
        ipfs_getter: IpfsGetter,
    ) -> Result<String, Error> {
        let connection_string = "host=localhost user=postgres password=postgres";
        let (mut client, connection) = tokio_postgres::connect(connection_string, NoTls).await?;
        tokio::spawn(connection);
        let transaction = client.transaction().await?;
        transaction.batch_execute(DB_SCHEMA).await?;

        let change_sets = event_log_to_change_sets(event_log, ipfs_getter);
        pin_mut!(change_sets);

        while let Some(change_set) = change_sets.next().await {
            transaction.simple_query(&change_set.sql).await?;
        }

        let rows = transaction
            .query("SELECT JSON_AGG(project) #>> '{}' FROM project;", &[])
            .await?;
        Ok(rows[0].get(0))
    }
}
