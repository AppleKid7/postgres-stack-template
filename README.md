# Postgres + pgAdmin (Docker Compose)

This stack runs a pinned Postgres database and a pgAdmin web UI.

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

## Connecting to Postgres

### Undockerized apps (host machine)

Connect to `localhost:5432`:

```
postgresql://<PG_USER>:<PG_PASSWORD>@localhost:5432/<db>
```

After a fresh init the server has two databases:

- `postgres` — the default database that always exists.
- `<PG_USER>` — a database the official image auto-creates, named after
  `POSTGRES_USER`.

Use whichever you prefer; no other databases are created.

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

Then connect to host **`postgres`** on port `5432`:

```
postgresql://<PG_USER>:<PG_PASSWORD>@postgres:5432/<db>
```

## pgAdmin

Open <http://localhost:8888> and log in with the `PGADMIN_DEFAULT_EMAIL` /
`PGADMIN_DEFAULT_PASSWORD` values from `.env`.

A server named `postgres` is already registered for you under **Servers**. It
connects to the `postgres` Docker service (the correct host from inside the
pgAdmin container), using `<PG_USER>` as the username.

> Important: inside the pgAdmin container, `localhost` is the pgAdmin container
> itself, not the Postgres server. The pre-registered server uses the Docker
> service name `postgres` as the host, so you don't need to register it manually.

The first time you click the `postgres` server, pgAdmin will ask for the
password. Enter your `<PG_PASSWORD>` value from `.env` and choose whether to
save it for future sessions.

## Common commands

```bash
docker compose up -d        # start in background
docker compose ps           # show running containers
docker compose logs -f      # tail logs
docker compose down         # stop and remove containers (volumes kept)
docker compose down -v       # also delete pg_data and pgadmin_data volumes
```