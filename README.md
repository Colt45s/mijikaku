# mijikaku

[URL Shortener Tutorial](https://docs.shuttle.rs/tutorials/url-shortener) Written in [Axum](https://github.com/tokio-rs/axum)

Deployed on [Shuttle](https://www.shuttle.rs/)

## Usage

```bash
$ curl -X POST -H "Content-Type: application/json" -d '{"url":"https://www.youtube.com/"}' https://mijikaku.shuttleapp.rs
```

Then, Copy the shortened URL and paste it in your browser.
