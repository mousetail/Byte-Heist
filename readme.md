# Project Overview

Byte Heist conists of two main projects

- The lang runner provides an isolated environment for langugaes, it runs inside docker/podman
- The main server provides the frontend. It should be able to run directly inside Linux or WSL.
  - Windows might work, but is untested

## Running the lang Runner

### Start up

```bash
docker compose up
```

### After updating the runner code

```bash
make restart-runner
```

## Starting the main server

### First time setup

First create a `.env.local` file with the following contents:

```
GITHUB_CLIENT_ID=
GITHUB_CLIENT_SECRET=

# The discord integration is optional, it should work fine without these vars set
DISCORD_WEBHOOK_URL=
DISCORD_TOKEN=
DISCORD_CHANNEL_ID=
```

Then create the datbase structure: (The database runs via the docker compose)

```bash
# Setup the development database
cargo install sqlx-cli --no-default-features --features rustls,postgres
sqlx migrate run
```

### Local Development

First, update the typescript definitions: (Optional, but allows the challenge editor to work)

```bash
npm install  # install typescript compiler
make ts-build-runner
```

This creates typescript definition files used for the challenge ditor.

Now run Vite to build the JS:

```bash
npx vite
```

This should just run in the background as long as you are working on the main server.

Finally, you can start the main server:

```
cargo run --bin main-server
```

Ensure the postgres is running since it checks the schema at compile time.

Now you should be able to visit Byte Heist at http://localhost:3001
