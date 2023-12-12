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
        .map(|event: &Event| -> ChangeSet { event_to_changeset(&event, ipfs_getter) })
        .collect();

    (change_sets, next_event_index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_log::InMemoryEventLog;

    const MAX_EVENTS_CHUNK_SIZE: usize = 2;

    fn dummy_ipfs_getter(_cid: &String) -> String {
        return "".to_string();
    }

    #[test]
    fn test_process_event_log_chunk_returns_change_sets() {
        let mut log = InMemoryEventLog { events: Vec::new() };
        log.append(Event {
            chain_id: 1,
            address: "0x123".to_string(),
            block_number: 4242,
            payload: EventPayload::ProjectCreated {
                project_id: "proj-123".to_string(),
            },
        });
        log.append(Event {
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

        let start = 0;
        // TODO make chunk size explicit; make process_event_log_chunk async
        let ipfs_getter = |_cid: &String| -> String { "{ \"foo\": \"bar\" }".to_string() };
        let (change_sets, _) =
            process_event_log_chunk(&log, start, MAX_EVENTS_CHUNK_SIZE, ipfs_getter);

        assert_eq!(
            change_sets[0].sql,
            r#"INSERT INTO "project" ("chain_id", "project_id", "created_at_block") VALUES (1, 'proj-123', 4242)"#
        );
        assert_eq!(
            change_sets[1].sql,
            r#"UPDATE "project" SET "metadata" = E'{ \"foo\": \"bar\" }' WHERE "chain_id" = 1 AND "project_id" = 'proj-123'"#
        )
    }

    #[tokio::test]
    async fn test_project_created() -> Result<(), Error> {
        let ipfs_getter = |_cid: &String| -> String { "".to_string() };
        let events = vec![Event {
            chain_id: 1,
            address: "0x123".to_string(),
            block_number: 4242,
            payload: EventPayload::ProjectCreated {
                project_id: "proj-123".to_string(),
            },
        }];

        let connection_string = "host=localhost user=postgres password=postgres";
        let (mut client, connection) = tokio_postgres::connect(connection_string, NoTls).await?;
        tokio::spawn(connection);
        let transaction = client.transaction().await?;
        transaction.batch_execute(DB_SCHEMA).await?;
        for event in events {
            let change_set = event_to_changeset(&event, ipfs_getter);
            transaction.simple_query(&change_set.sql).await?;
        }
        let rows = transaction
            .query("SELECT JSON_AGG(project) #>> '{}' FROM project;", &[])
            .await?;
        let value: &str = rows[0].get(0);

        insta::assert_yaml_snapshot!(value);
        Ok(())
    }

    #[tokio::test]
    async fn test_project_created_and_metadata_updated() -> Result<(), Error> {
        let ipfs_getter = |_cid: &String| -> String { "{ \"foo\": \"bar\" }".to_string() };
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

        let connection_string = "host=localhost user=postgres password=postgres";
        let (mut client, connection) = tokio_postgres::connect(connection_string, NoTls).await?;
        tokio::spawn(connection);
        let transaction = client.transaction().await?;
        transaction.batch_execute(DB_SCHEMA).await?;
        for event in events {
            let change_set = event_to_changeset(&event, ipfs_getter);
            transaction.simple_query(&change_set.sql).await?;
        }
        let rows = transaction
            .query("SELECT JSON_AGG(project) #>> '{}' FROM project;", &[])
            .await?;
        let value: &str = rows[0].get(0);

        insta::assert_yaml_snapshot!(value);
        Ok(())
    }
}
