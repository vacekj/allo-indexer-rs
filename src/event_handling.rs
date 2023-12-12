use sea_query::{Expr, Iden, PostgresQueryBuilder, Query};
use tokio_postgres::{Client, Error, NoTls, Transaction};

const DB_SCHEMA: &str = r#"
   CREATE TABLE project (
     chain_id INTEGER NOT NULL,
     project_id VARCHAR NOT NULL,
     created_at_block BIGINT NOT NULL,
     metadata JSONB,
     PRIMARY KEY(chain_id, project_id)
   );
"#;

#[derive(Iden)]
enum Project {
    Table,
    ChainId,
    ProjectId,
    CreatedAtBlock,
    Metadata,
    Owners,
}

// TODO use stricter types
pub struct Event {
    pub chain_id: i32,
    pub address: String,
    pub block_number: i32,
    pub payload: EventPayload,
}

pub enum EventPayload {
    ProjectCreated {
        project_id: String,
    },
    MetadataUpdated {
        project_id: String,
        meta_ptr: MetaPtr,
    },
    OwnerAdded {
        project_id: String,
        owner: String,
    },
    OwnerRemoved {
        project_id: String,
        owner: String,
    },
}

pub struct MetaPtr {
    pointer: String,
}

pub type IpfsGetter = fn(String) -> String;

pub struct ChangeSet {
    sql: String,
    // TODO add other effects here, e.g. subscriptions: ...
}

pub fn event_to_changeset(event: Event, ipfs: IpfsGetter) -> ChangeSet {
    match event.payload {
        EventPayload::ProjectCreated { project_id } => ChangeSet {
            sql: Query::insert()
                .into_table(Project::Table)
                .columns([
                    Project::ChainId,
                    Project::ProjectId,
                    Project::CreatedAtBlock,
                ])
                .values_panic([
                    event.chain_id.into(),
                    project_id.into(),
                    event.block_number.into(),
                ])
                .to_string(PostgresQueryBuilder),
        },

        EventPayload::MetadataUpdated {
            meta_ptr,
            project_id,
        } => {
            let metadata = ipfs(meta_ptr.pointer);
            ChangeSet {
                sql: Query::update()
                    .table(Project::Table)
                    .values([(Project::Metadata, metadata.into())])
                    .and_where(Expr::col(Project::ChainId).eq(event.chain_id))
                    .and_where(Expr::col(Project::ProjectId).eq(project_id))
                    .to_string(PostgresQueryBuilder),
            }
        }

        EventPayload::OwnerAdded { project_id, owner } => {
            ChangeSet {
                // TODO build SQL safely
                sql: format!(
                    r#"UPDATE "project" SET "owners" = ("owners" || '["{}"]') WHERE "chain_id" = {} AND "project_id" = '{}'"#,
                    owner, event.chain_id, project_id
                ),
            }
        }

        EventPayload::OwnerRemoved { project_id, owner } => {
            ChangeSet {
                // TODO build SQL safely
                sql: format!(
                    r#"UPDATE "project" SET "owners" = ("owners" - '{}') WHERE "chain_id" = {} AND "project_id" = '{}'"#,
                    owner, event.chain_id, project_id
                ),
            }
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    fn dummy_ipfs_getter(_cid: String) -> String {
        return "".to_string();
    }

    #[test]
    fn test_handle_project_created() {
        let event = Event {
            chain_id: 1,
            address: "0x123".to_string(),
            block_number: 4242,
            payload: EventPayload::ProjectCreated {
                project_id: "proj-123".to_string(),
            },
        };

        assert_eq!(
            event_to_changeset(event, dummy_ipfs_getter).sql,
            r#"INSERT INTO "project" ("chain_id", "project_id", "created_at_block") VALUES (1, 'proj-123', 4242)"#
        );
    }

    #[test]
    fn test_handle_metadata_updated() {
        let event = Event {
            chain_id: 1,
            address: "0x123".to_string(),
            block_number: 4242,
            payload: EventPayload::MetadataUpdated {
                project_id: "proj-123".to_string(),
                meta_ptr: MetaPtr {
                    pointer: "123".to_string(),
                },
            },
        };
        let ipfs_getter = |_cid: String| -> String { r#"{ "foo": "bar" }"#.to_string() };

        assert_eq!(
            event_to_changeset(event, ipfs_getter).sql,
            r#"UPDATE "project" SET "metadata" = E'{ \"foo\": \"bar\" }' WHERE "chain_id" = 1 AND "project_id" = 'proj-123'"#
        );
    }

    #[test]
    fn test_handle_owner_added() {
        let event = Event {
            chain_id: 1,
            address: "0x123".to_string(),
            block_number: 4242,
            payload: EventPayload::OwnerAdded {
                project_id: "proj-123".to_string(),
                owner: "0x123".to_string(),
            },
        };

        assert_eq!(
            event_to_changeset(event, dummy_ipfs_getter).sql,
            r#"UPDATE "project" SET "owners" = ("owners" || '["0x123"]') WHERE "chain_id" = 1 AND "project_id" = 'proj-123'"#
        );
    }

    #[test]
    fn test_handle_owner_removed() {
        let event = Event {
            chain_id: 1,
            address: "0x123".to_string(),
            block_number: 4242,
            payload: EventPayload::OwnerRemoved {
                project_id: "proj-123".to_string(),
                owner: "0x123".to_string(),
            },
        };

        assert_eq!(
            event_to_changeset(event, dummy_ipfs_getter).sql,
            r#"UPDATE "project" SET "owners" = ("owners" - '0x123') WHERE "chain_id" = 1 AND "project_id" = 'proj-123'"#
        );
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    async fn snapshot_test_events(
        events: Vec<Event>,
        ipfs_getter: IpfsGetter,
    ) -> Result<(), Error> {
        let connection_string = "host=localhost user=postgres password=postgres";
        let (mut client, connection) = tokio_postgres::connect(connection_string, NoTls).await?;
        tokio::spawn(connection);
        let transaction = client.transaction().await?;
        transaction.batch_execute(DB_SCHEMA).await?;
        for event in events {
            let change_set = event_to_changeset(event, ipfs_getter);
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
    async fn test_project_created() -> Result<(), Error> {
        let ipfs_getter = |_cid: String| -> String { "".to_string() };
        let events = vec![Event {
            chain_id: 1,
            address: "0x123".to_string(),
            block_number: 4242,
            payload: EventPayload::ProjectCreated {
                project_id: "proj-123".to_string(),
            },
        }];

        snapshot_test_events(events, ipfs_getter).await?
    }

    #[tokio::test]
    async fn test_project_created_and_metadata_updated() -> Result<(), Error> {
        let ipfs_getter = |_cid: String| -> String { "{ \"foo\": \"bar\" }".to_string() };
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
            let change_set = event_to_changeset(event, ipfs_getter);
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
