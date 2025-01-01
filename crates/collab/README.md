# Editsync Server

This crate is what we run at https://collab.editsync.khulnasoft.com.

It contains our back-end logic for collaboration, to which we connect from the Editsync client via a websocket after authenticating via https://editsync.khulnasoft.com, which is a separate repo running on Vercel.

# Local Development

## Database setup

Before you can run the collab server locally, you'll need to set up a editsync Postgres database. Follow the steps sequentially:

1. Ensure you have postgres installed. If not, install with `brew install postgresql@15`.
2. Follow the steps on Brew's formula and verify your `$PATH` contains `/opt/homebrew/opt/postgresql@15/bin`.
3. If you hadn't done it before, create the `postgres` user with `createuser -s postgres`.
4. You are now ready to run the `bootstrap` script:

```sh
script/bootstrap
```

This script will set up the `editsync` Postgres database, and populate it with some users. It requires internet access, because it fetches some users from the GitHub API.

The script will create several _admin_ users, who you'll sign in as by default when developing locally. The GitHub logins for the default users are specified in the `seed.default.json` file.

To use a different set of admin users, create `crates/collab/seed.json`.

```json
{
  "admins": ["yourgithubhere"],
  "channels": ["editsync"]
}
```

## Testing collaborative features locally

In one terminal, run Editsync's collaboration server and the livekit dev server:

```sh
foreman start
```

In a second terminal, run two or more instances of Editsync.

```sh
script/editsync-local -2
```

This script starts one to four instances of Editsync, depending on the `-2`, `-3` or `-4` flags. Each instance will be connected to the local `collab` server, signed in as a different user from `seed.json` or `seed.default.json`.

# Deployment

We run two instances of collab:

- Staging (https://staging-collab.editsync.khulnasoft.com)
- Production (https://collab.editsync.khulnasoft.com)

Both of these run on the Kubernetes cluster hosted in Digital Ocean.

Deployment is triggered by pushing to the `collab-staging` (or `collab-production`) tag in Github. The best way to do this is:

- `./script/deploy-collab staging`
- `./script/deploy-collab production`

You can tell what is currently deployed with `./script/what-is-deployed`.

# Database Migrations

To create a new migration:

```sh
./script/create-migration <name>
```

Migrations are run automatically on service start, so run `foreman start` again. The service will crash if the migrations fail.

When you create a new migration, you also need to update the [SQLite schema](./migrations.sqlite/20221109000000_test_schema.sql) that is used for testing.