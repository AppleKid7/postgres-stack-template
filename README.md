# Postgres + pgvector + pgAdmin (Docker Compose)

This stack runs two independent PostgreSQL 18 instances (one regular, one with
pgvector installed) and a pgAdmin web UI. Each service has its own persistent
volume. Published ports are bound to `127.0.0.1` for access from this machine.

## Prerequisites

Create a `.env` file (next to this `docker-compose.yml`) with:

```
PG_USER=<your db user>
PG_PASSWORD=<your db password>
PGADMIN_DEFAULT_EMAIL=<email-shaped login for the pgAdmin UI>
PGADMIN_DEFAULT_PASSWORD=<password for the pgAdmin UI>
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
docker compose down -v       # also delete pg_data, pgv_data, and pgadmin_data (all stored data)
```
