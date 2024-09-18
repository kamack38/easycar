# EasyCar

This is a repo for easy car - an app which finds the newest exams on [info-car.pl](https://info-car.pl/)

**Remember to give a ⭐.**

## Features

- [x] Username and password login
- [x] Session refreshing
- [x] Separate library crate
- [x] Enrolling to exam
- [x] Easy deployment to shuttle.rs
- [x] Mobile notifications via a telegram
- [x] Customisable telegram bot 
- [x] Checking exam status
- [x] Paying for exam using BLIK code

## Prerequisites

To run this project and use the telegram bot you have to fill in the `Secrets.toml.example` file and then rename it to `Secrets.toml`.

You can create your own bot using the [BotFather](https://t.me/botfather).

## Installing

```bash
cargo install --git https://github.com/kamack38/easycar.git
```

## Building

```bash
cargo build --release --bin easycar-service
```

## Running

```bash
RUST_LOG="INFO" ./target/release/easycar-service
```

## Deployment

This repo provides a [shuttle](shuttle.rs) deployment.

To use it create an account at [shuttle](https://console.shuttle.rs/login).

Install `cargo-shuttle`:

```bash
cargo install cargo-shuttle
```

and login to your account:

```bash
cargo shuttle login
```

To deploy run:

```bash
cargo shuttle deploy
```

By default shuttle puts this project to sleep after 30 minutes to disable this run:

```bash
cargo-shuttle project restart --idle-minutes 0
```

## License

The `easycar` project is distributed under the AGPL-3.0 license and the `info-car-api` crate is distributed under the LGPL-3.0 license. The license files are stored in the respective project roots.

Copyright (C) 2024  Kamack38

Made with :heart: in :poland:!
