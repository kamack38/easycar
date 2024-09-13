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
- [ ] /enroll autocomplete
- [ ] Custom keyboards

## Prerequisites

To run this project and use the telegram bot you have to fill in the `Secrets.toml.example` file and then rename it to `Secrets.toml`.

You can create your own bot using the [BotFather](https://t.me/botfather).

## Running 

```bash
cargo run
```

## Building

```bash
cargo build --release
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

Made with :heart: in :poland:!
