use crate::{
    event_handling::{
        event_to_changeset, ChangeSet, Event, EventPayload, IpfsGetter, MetaPtr, DB_SCHEMA,
    },
    event_log::EventLog,
};
use sea_query::{Expr, Iden, PostgresQueryBuilder, Query};
use tokio_postgres::{Client, Error, NoTls, Transaction};

fn process_event_log_chunk(
    log: &dyn EventLog,
    start: usize,
    max_events_chunk_size: usize,
    ipfs_getter: IpfsGetter,
) -> (Vec<ChangeSet>, usize) {
    let (events, next_event_index) = log.range(0, start + max_events_chunk_size);

    let change_sets: Vec<ChangeSet> = events
        .iter()
        .map(|event: &Event| -> ChangeSet { event_to_changeset(event, ipfs_getter) })
        .collect();

    (change_sets, next_event_index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_log::InMemoryEventLog;

    const MAX_EVENTS_CHUNK_SIZE: usize = 2;

    fn dummy_ipfs_getter(_cid: &str) -> String {
        "".into()
    }

    #[tokio::test]
    async fn test_project_created() {
        let ipfs_getter = |_cid: &str| -> String { "".to_string() };
        let events =
            vec![Event {
                chain_id: 1,
                address: "0x123".to_string(),
                block_number: 4242,
                payload: EventPayload::ProjectCreated {
                    project_id: "proj-123".to_string(),
                },
            }];
        let value = snapshot_test_events(events, ipfs_getter).await;

        insta::assert_yaml_snapshot!(value.unwrap());
    }

    #[tokio::test]
    async fn test_project_created_and_metadata_updated() {
        let ipfs_getter = |_cid: &str| -> String { "{ \"foo\": \"bar\" }".to_string() };
        let events = vec![
            Event {
                chain_id: 1,
                address: "0x123".to_string(),
                block_number: 4242,
                payload: EventPayload::ProjectCreated {
                    project_id: "proj-123".to_string(),
                },
            },
            Event {
                chain_id: 1,
                address: "0x123".to_string(),
                block_number: 4242,
                payload: EventPayload::MetadataUpdated {
                    project_id: "proj-123".to_string(),
                    meta_ptr: MetaPtr {
                        pointer: "123".to_string(),
                    },
                },
            },
        ];
        let value = snapshot_test_events(events, ipfs_getter).await;

        insta::assert_yaml_snapshot!(value.unwrap());
    }

    async fn snapshot_test_events(
        events: Vec<Event>,
        ipfs_getter: IpfsGetter,
    ) -> Result<String, Error> {
        let mut event_log = InMemoryEventLog::new();
        for event in events {
            event_log.append(event);
        }
        let connection_string = "host=localhost user=postgres password=postgres";
        let (mut client, connection) = tokio_postgres::connect(connection_string, NoTls).await?;
        tokio::spawn(connection);
        let transaction = client.transaction().await?;
        transaction.batch_execute(DB_SCHEMA).await?;
        let (change_sets, _) =
            process_event_log_chunk(&event_log, 0, MAX_EVENTS_CHUNK_SIZE, ipfs_getter);
        for change in change_sets {
            transaction.simple_query(&change.sql).await?;
        }
        let rows = transaction
            .query("SELECT JSON_AGG(project) #>> '{}' FROM project;", &[])
            .await?;
        Ok(rows[0].get(0))
    }
}
