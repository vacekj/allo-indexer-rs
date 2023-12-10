## ProjectCreated

Event:

```json
{
  "chainId": 1,
  "type": "ProjectCreated",
  "address": "0x123",
  "blockNumber": 12345,
  "args": {
    "projectID": "proj-123",
    "owner": "0x123"
  }
}
```

Change:

```sql
INSERT INTO project (chain_id, project_id, created_at_block, metadata)
VALUES (
  '<event.chainId>',
  '<event.args.projectID>',
  '<event.blockNumber>',
  NULL
);
```

## MetadataUpdated

Event:

```json
{
  "chainId": 1,
  "type": "MetadataUpdated",
  "address": "0x123",
  "blockNumber": 12345,
  "args": {
    "projectID": "proj-123",
    "metaPtr": { "pointer": "proj-123" }
  }
}
```

Change:

```sql
UPDATE project
SET metadata = '<ipfs(event.args.metaPtr.pointer)>'
WHERE project.chain_id = '<event.chainId>'
AND project.project_id = '<event.args.projectID>';
```

## OwnerAdded

Event:

```json
{
  "chainId": 1,
  "type": "OwnerAdded",
  "address": "0x123",
  "blockNumber": 12345,
  "args": {
    "projectID": "proj-123",
    "owner": "0x123"
  }
}
```

Change:

```sql
UPDATE project
SET owners = (owners || '["<event.args.owner>"]')
WHERE project.chain_id = <event.chainId>
AND project.project_id = '<event.args.projectID>';
```

## OwnerRemoved

```json
{
  "chainId": 1,
  "type": "OwnerRemoved",
  "address": "0x123",
  "blockNumber": 12345,
  "args": {
    "projectID": "proj-123",
    "owner": "0x123"
  }
}
```

Change:

```sql
UPDATE project
SET owners = (owners - '<event.args.owner>')
WHERE project.chain_id = <event.chainId>
AND project.project_id = '<event.args.projectID>';
```
