use ethers::providers::admin::ProtocolInfo;
use sea_orm::ConditionalStatement;
use sea_query::{Expr, Iden, PostgresQueryBuilder, Query};

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
    chain_id: i32,
    address: String,
    block_number: i32,
    payload: EventPayload,
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
pub fn handle_event<IpfsGetter: Fn(String) -> String>(event: Event, ipfs: IpfsGetter) -> String {
    match event.payload {
        EventPayload::ProjectCreated { project_id } => Query::insert()
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

        EventPayload::MetadataUpdated {
            meta_ptr,
            project_id,
        } => {
            let metadata = ipfs(meta_ptr.pointer);
            println!("Value: {}", metadata.to_string());
            Query::update()
                .table(Project::Table)
                .values([(Project::Metadata, metadata.into())])
                .and_where(Expr::col(Project::ChainId).eq(event.chain_id))
                .and_where(Expr::col(Project::ProjectId).eq(project_id))
                .to_string(PostgresQueryBuilder)
        }

        EventPayload::OwnerAdded { project_id, owner } => {
            // TODO use query builder if possible
            // TODO build SQL safely
            format!(
                r#"UPDATE "project" SET "owners" = ("owners" || '["{}"]') WHERE "chain_id" = {} AND "project_id" = '{}'"#,
                owner, event.chain_id, project_id
            )
        }

        EventPayload::OwnerRemoved { project_id, owner } => {
            // TODO use query builder if possible
            // TODO build SQL safely
            format!(
                r#"UPDATE "project" SET "owners" = ("owners" - '{}') WHERE "chain_id" = {} AND "project_id" = '{}'"#,
                owner, event.chain_id, project_id
            )
        }
    }
}

#[cfg(test)]
mod tests {
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
            handle_event(event, dummy_ipfs_getter),
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

        assert_eq!(
            handle_event(event, |_cid: String| -> String {
                r#"{ "foo": "bar" }"#.to_string()
            }),
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
            handle_event(event, dummy_ipfs_getter),
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
            handle_event(event, dummy_ipfs_getter),
            r#"UPDATE "project" SET "owners" = ("owners" - '0x123') WHERE "chain_id" = 1 AND "project_id" = 'proj-123'"#
        );
    }
}
