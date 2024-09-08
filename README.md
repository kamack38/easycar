# EasyCar

This is a repo for easy car - an app which finds the newest exams on [info-car.pl](https://info-car.pl/)

## Running 

Copy the `.env.example` file to `.env` and fill the `TOKEN` var with your Bearer token

```bash
cargo run
```

## Building

```bash
cargo build --release
```

Then place all required vars in a `.env` file 

## TODO

- [x] Username and password login
- [x] Session refreshing
- [x] Separate library crate
- [x] Phone notifications via a webhook
- [x] Exam signing
- [x] Automatic token refreshing
- [ ] Retry token refreshing
- [ ] Refactor telegram bot to use dispatching
- [ ] Better error handeling
- [ ] Easy deploy to shuttle.rs
- [ ] Custom keyboard for signing for an exam with a single click
