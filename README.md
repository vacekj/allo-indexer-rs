# allo-indexer-rs
Index the Allo protocol on-chain events blazingly fast

# Architecture
- The only storage backend for the whole indexer is a single PostgreSQL db. We use it to store both typical values, JSON & use it as a key-value store
- Token prices are indexed from on-chain AMMs, no need for an external service
- This also obviates the need for block time estimates.
- Passport is ingested in batch mode
- The only dependency is an RPC and a DB connection
- We use Supabase as the default host for the DB, making the data easily publicly queryable
- All of the data in the database is public, allowing for a public connection credential for general access
- Blocks, IPFS data and prices are immutable, and we can cache them inside of Postgres
- We can restore postgres dumps of the immutable data do spin up a new instance instantly
- We index chains in parallel
- We fetch block ranges and IPFS files in parallel
- We index per-round, making the initial startup time almost instant
- HTTP API is fully decoupled from the indexer workers, allowing changes without redeploys
- Basic QF calculations (without overrides or estimates) can be done in-database as a materialized view and automatically updated and cached
- We use Kani for verification of the correctness of the QF and QV algorithms
