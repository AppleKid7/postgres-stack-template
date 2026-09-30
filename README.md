# Postgres + pgvector + pgAdmin + SpacetimeDB + Stargate (Docker Compose)

This stack runs two independent PostgreSQL 18 instances (one regular, one with
pgvector installed), a pgAdmin web UI, a SpacetimeDB server, and a browser-based
Stargate desktop client. Each service has its own persistent volume. Published
ports are bound to `127.0.0.1` for access from this machine.

## Prerequisites

Create a `.env` file (next to this `docker-compose.yml`) with:

```
PG_USER=<your db user>
PG_PASSWORD=<your db password>
PGADMIN_DEFAULT_EMAIL=<email-shaped login for the pgAdmin UI>
PGADMIN_DEFAULT_PASSWORD=<password for the pgAdmin UI>
STARGATE_USER=<username for Stargate browser auth>
STARGATE_PASSWORD=<password for Stargate browser auth>
```

The compose file reads these via variable interpolation. The compose project is
named `postgres` (see `name:` at the top of `docker-compose.yml`), so the
user-defined bridge network is published as **`postgres_dbnet`** — that exact
name is what external apps must reference.

## First-time setup

From this directory, validate and start the stack:

```bash
docker compose config --quiet
docker compose up -d
docker compose ps
```

Follow the pgAdmin setup below to check the server registrations, save connection
passwords, and verify that the vector extension was initialized.

## Connecting to Postgres

### Undockerized apps (host machine)

Use these host-side addresses:

| Instance | Address |
|---|---|
| Regular PostgreSQL | `127.0.0.1:5432` |
| PostgreSQL with pgvector | `127.0.0.1:5434` |

```
postgresql://<PG_USER>:<PG_PASSWORD>@127.0.0.1:5432/<db>
postgresql://<PG_USER>:<PG_PASSWORD>@127.0.0.1:5434/<db>
```

After a fresh initialization, each instance has these non-template databases:

- `postgres` — the default database that always exists.
- `<PG_USER>` — a database the official image auto-creates, named after
  `POSTGRES_USER`.

If `PG_USER` is `postgres`, these refer to the same database. On the pgvector
instance, only the `postgres` database has the vector extension enabled
automatically.

### Dockerized apps (Django / Phoenix in their own compose file)

Add the network to the app's own `docker-compose.yml`:

```yaml
networks:
  dbnet:
    external: true
    name: postgres_dbnet
```

Attach the app's service to it:

```yaml
services:
  web:
    # ...
    networks:
      - dbnet
```

Then connect to host **`postgres`** or **`pgvector`**, both on port `5432`:

```
postgresql://<PG_USER>:<PG_PASSWORD>@postgres:5432/<db>
postgresql://<PG_USER>:<PG_PASSWORD>@pgvector:5432/<db>
```

## pgAdmin

Open <http://127.0.0.1:8888> and log in with the `PGADMIN_DEFAULT_EMAIL` /
`PGADMIN_DEFAULT_PASSWORD` values from `.env`.

### Check or add server registrations

On a fresh `pgadmin_data` volume, both server definitions are imported from the
Compose configuration. With an existing volume, pgAdmin retains its previous
registrations, so you may see only `postgres`. Running containers do not
automatically appear in the server list.

To add the missing second server:

1. Right-click **Servers** → **Register** → **Server…**.
2. On the **General** tab, set **Name** to `pgvector`.
3. On the **Connection** tab, enter:

   | Field | Value |
   |---|---|
   | Host name/address | `pgvector` |
   | Port | `5432` |
   | Maintenance database | `postgres` |
   | Username | Your `PG_USER` value from `.env` |
   | Password | Your `PG_PASSWORD` value from `.env` |
   | Save password | Enabled |

4. Click **Save**.

The regular PostgreSQL registration uses the same settings except its name and
host are `postgres`. If either registration already exists, check its settings
under **Properties** rather than adding a duplicate.

Inside pgAdmin, use Docker service names and port **5432 for both servers**.
`127.0.0.1` would refer to the pgAdmin container itself, and `5434` is only the
pgvector instance's published host port.

When connecting to either server for the first time, enter `PG_PASSWORD` and
enable **Save password**. Registrations and saved passwords persist in
`pgadmin_data`. Server definitions are not replaced on each startup; later edits
to the embedded `servers.json` do not automatically update existing registrations.

### Verify pgvector

Under the **pgvector** server, select **Databases → postgres** and open the
**Query Tool**. Run:

```sql
SELECT extname, extversion
FROM pg_extension
WHERE extname = 'vector';
```

With the currently pinned image, expect one row showing `vector` and `0.8.6`.

The mounted `init/pgvector/01-enable-vector.sql` script connects to the `postgres`
database and enables the extension during the first initialization of `pgv_data`.
Initialization scripts do not run again on an already-initialized volume. If the
query returns no rows, confirm you are connected to the pgvector instance, then
run the following as `PG_USER` and repeat the check:

```sql
CREATE EXTENSION IF NOT EXISTS vector;
```

### Enable pgvector in future databases

Create the database on the **pgvector instance**, connect to that database, and
run `CREATE EXTENSION IF NOT EXISTS vector;` as a sufficiently privileged role
(such as `PG_USER`). Extensions are enabled per database; enabling one in
`postgres` does not automatically enable it in future databases. No container
restart is needed.

The regular `postgres` instance does not include the vector extension files.

### Existing data and credentials

`POSTGRES_USER` and `POSTGRES_PASSWORD` initialize new database volumes; changing
their `.env` values does not change roles or passwords in an existing database.
Similarly, pgAdmin's default login credentials initialize its first administrator
account rather than resetting an existing account on each startup.

## Common commands

```bash
docker compose up -d        # start in background
docker compose ps           # show running containers
docker compose logs -f      # tail logs
docker compose down         # stop and remove containers (volumes kept)
docker compose down -v       # also delete pg_data, pgv_data, pgadmin_data, spacetimedb_data, and stargate_config (all stored data)
```

## SpacetimeDB

SpacetimeDB is a real-time database for multiplayer games and applications.

### Access

| Client | URL |
|---|---|
| From this machine | `http://127.0.0.1:3333` |
| From containers on `dbnet` | `http://spacetimedb:3333` |

### CLI access (containerized)

You do not need to install the SpacetimeDB CLI on your machine. Use the CLI
from within a temporary container:

```bash
docker run --rm -it --network postgres_dbnet clockworklabs/spacetime:v2.10.2 \
  server add --url http://spacetimedb:3333 local
```

This registers the server with the nickname `local`. Subsequent commands use
`--server local` or set it as default.

### First-time setup: publish the sample database

The `stargate-demo/` directory contains a minimal Rust module with a `message`
table and a `send_message` reducer. To publish it:

1. Build the module in a temporary container:

   ```bash
   docker run --rm -v ./stargate-demo:/app -w /app/spacetimedb rust:1.84 \
     cargo build --release --target wasm32-unknown-unknown
   ```

   This produces `stargate-demo/spacetimedb/target/wasm32-unknown-unknown/release/stargate_demo.wasm`.

2. Publish the module to your SpacetimeDB server:

   ```bash
   docker run --rm --network postgres_dbnet -v ./stargate-demo:/app -w /app \
     clockworklabs/spacetime:v2.10.2 publish --server local --anonymous stargate-demo
   ```

   The `--anonymous` flag creates an anonymous identity for this session. The
   database name `stargate-demo` is derived from the directory name.

3. Verify the database exists:

   ```bash
   docker run --rm --network postgres_dbnet clockworklabs/spacetime:v2.10.2 \
     list --server local --anonymous
   ```

   You should see `stargate-demo` in the output.

### Verify SpacetimeDB

Query the sample database:

```bash
docker run --rm --network postgres_dbnet clockworklabs/spacetime:v2.10.2 \
  sql --server local --anonymous stargate-demo "SELECT * FROM message"
```

Initially, the table is empty. Add a message:

```bash
docker run --rm --network postgres_dbnet clockworklabs/spacetime:v2.10.2 \
  call --server local --anonymous stargate-demo send_message "alice" "Hello, SpacetimeDB!"
```

Then query again to see the new row.

### Enable pgvector-style extensions in SpacetimeDB

SpacetimeDB uses WebAssembly modules for schema and logic. Extensions are not
applicable in the same way as PostgreSQL. Define tables and reducers in your
module code and publish updates with `spacetime publish`.

## Stargate

Stargate is a desktop GUI client for SpacetimeDB, packaged here as a
browser-accessible container.

### Access

Open <http://127.0.0.1:8889> in your browser. Log in with the `STARGATE_USER`
and `STARGATE_PASSWORD` values from `.env`.

### Connect to SpacetimeDB

1. In Stargate, click **Add Server** or **Connect**.
2. Enter the server URL: `http://spacetimedb:3333`
3. For authentication, use the same anonymous identity you created when
   publishing the module, or create a new one. Stargate will prompt you to
   sign in or enter a token.
4. Select the `stargate-demo` database.

### Verify Stargate features

- **Tables**: Browse the `message` table.
- **SQL**: Run `SELECT * FROM message` in the SQL console.
- **Logs**: Open the Logs view to see module output, including messages from
  the `send_message` reducer.
- **Live view**: Subscribe to `SELECT * FROM message` and watch rows update
  when you invoke `send_message` from the CLI.

Stargate's free tier provides read-only access. Saved connections and
credentials persist in `stargate_config`.

### Existing data and credentials

`STARGATE_USER` and `STARGATE_PASSWORD` configure HTTP Basic authentication
for the browser interface. Changing these values in `.env` updates the
credentials on the next container restart. Saved SpacetimeDB connections
persist in the `stargate_config` volume.

## Remote access from other machines

All published ports are bound to `127.0.0.1` for local-only access. To allow
access from other machines on your network, change the bindings to a specific
IP address or `0.0.0.0`.

### Direct network access

Replace `127.0.0.1` with the Docker host's LAN IP (e.g., `192.168.1.50`) or
`0.0.0.0` for all interfaces:

```yaml
# Example: bind to a specific LAN interface
ports:
  - "192.168.1.50:5432:5432"   # PostgreSQL
  - "192.168.1.50:5434:5432"   # pgvector
  - "192.168.1.50:8888:80"     # pgAdmin
  - "192.168.1.50:3333:3333"   # SpacetimeDB
  - "192.168.1.50:8889:3000"   # Stargate
```

**Important considerations:**

- Remote clients connect using the Docker host's IP and published port.
- pgAdmin and Stargate continue using Docker service names internally
  (`postgres`, `pgvector`, `spacetimedb`).
- Publish only the services remote clients need.
- Permit the relevant ports through your host firewall.
- PostgreSQL access may require adjusting `listen_addresses` and `pg_hba.conf`
  rules in the database configuration.
- For access over untrusted networks, use a VPN, SSH tunnel, or authenticated
  TLS reverse proxy. Web services (pgAdmin, SpacetimeDB, Stargate) require
  WebSocket support if proxied.

### SSH tunnel access (no binding changes)

If you prefer to keep local-only bindings, remote users can SSH-tunnel to the
Docker host:

```bash
# From a remote machine
ssh -L 5432:127.0.0.1:5432 \
    -L 5434:127.0.0.1:5434 \
    -L 8888:127.0.0.1:8888 \
    -L 3333:127.0.0.1:3333 \
    -L 8889:127.0.0.1:8889 \
    user@docker-host
```

Then connect to `localhost:<port>` on the remote machine as if the services
were running locally.
