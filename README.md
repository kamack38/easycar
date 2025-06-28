# EasyCar

This is a repo for easy car - an app which finds the newest exams on
[info-car.pl](https://info-car.pl/)

**Remember to give this repo a ⭐.**

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

To run this project and use the telegram bot you have to fill in the `Secrets.toml.example` file and
then rename it to `Secrets.toml`.

If you want to choose a proper `OSK_ID` for your exam centre you can see the corresponding word
names and `OKS_ID`s [here](./docs/WORD-mapping.md).

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

## Running as a daemon

A user systemd unit file is provided (`assets/easycar.service`). `Secrets.toml` needs to reside in
the home directory of a user. After completing the installation, copy the unit file to
`~/.config/systemd/user/` and then enable and start the service:

```bash
systemctl --user enable --now easycar.service
```

If you want it to start automatically after a reboot, enable lingering for your user:

```bash
sudo loginctl enable-linger "$(whoami)"
```

## Deployment

This repo provides a [shuttle](https://shuttle.dev) deployment.

To use it create an account at [shuttle.dev](https://console.shuttle.dev/signup).

Install `cargo-shuttle`:

```bash
cargo install cargo-shuttle
```

and login to your account:

```bash
shuttle login
```

Create a new project:

```bash
shuttle init
```

To deploy run:

```bash
shuttle deploy
```

## Roadmap

I consider the project feature complete but if You are missing certain a feature please contribute.

The only thing I may refactor is the logining in, because it currently depends on a bug on the
website.

## License

The `easycar` project is distributed under the AGPL-3.0 license and the `info-car-api` crate is
distributed under the LGPL-3.0 license. The license files are stored in the respective project
roots.

Copyright (C) 2024 Kamack38

Made with :heart: in :poland:!
