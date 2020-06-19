# Corgo Bot

A discord bot for the [Party Corgi Network Discord](https://www.partycorgi.com/).

The bot is split into two instances: a [JS](./js-bot) implementation and a [Rust](./rust-bot) implementation.

## Working on the Bot

You'll need an env var named `DISCORD_TOKEN` with a discord bot token in it. If you're a Maintainer in the PCN Discord you can ask for the production token to test with (in the future we'll use 1password). If you don't have the Maintainer role you may have to set up your own test server and bot token.

TODO: explain how to to the above token stuffs.

## Working on JS Bot

```
cd js-bot
yarn
yarn develop
```

## Working on Rust Bot

```
cd rust-bot
cargo run
```

## Deployment

Both instances of the bot are located entirely in their respective directories. They contain language-specific tooling (cargo, yarn, etc) and a `Dockerfile` [[1](./rust-bot/Dockerfile), [2](./js-bot/Dockerfile)] for production.

A GitHub Action (in [`./github/workflows`](./.github/workflows)) is responsible for deploying changes for each instance of the bot. The workflows run automatically on merge to master, upload .zips of the relevant docker image, and only run on changes to the relevant files (A change to the JS bot _does not_ trigger a deployment for the Rust bot).
