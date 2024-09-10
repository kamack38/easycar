# EasyCar

This is a repo for easy car - an app which finds the newest exams on [info-car.pl](https://info-car.pl/)

## Prerequisites

Do be able to run this project and use the telegram bot you have to fill in the `Secrets.toml.example` file and then rename it to `Secrets.toml`.

You can create your own bot using the [BotFather](https://t.me/botfather).

## Running 

```bash
cargo run
```

## Building

```bash
cargo build --release
```

Then place all required vars in a `.env` file 

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

## TODO

- [x] Username and password login
- [x] Session refreshing
- [x] Separate library crate
- [x] Phone notifications via a webhook
- [x] Exam signing
- [x] Automatic token refreshing
- [ ] Retry token refreshing
- [ ] Refactor telegram bot to use dispatching
- [ ] Better error handling
- [ ] Easy deploy to shuttle.rs
- [ ] Custom keyboard for signing for an exam with a single click
