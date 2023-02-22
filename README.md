# resentral_server_rs
Re-write and basically revival of my old Sentral web-scraping server for resentral. The main app is at [this repo](https://github.com/jumpyjacko/resentral_flutter).

### About how I made this
This is a Rust based server written using tokio's `axum` (at [this repo](https://github.com/tokio-rs/axum)) with web-scraping functionality from `fantoccini` (at [this repo](https://github.com/jonhoo/fantoccini)). Its not exactly the best server, but its definitely a server. This is also my first time using Docker so excuse the bad `Dockerfile`. Possibly (most likely) unsafe and that's okay.

### How to use
Its Docker, just build the Docker image, or better yet, go to Docker hub and search for `resentral-server`.

### Contributing
Make changes, pr it, I'll probably look at it.
