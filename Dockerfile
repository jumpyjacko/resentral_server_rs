FROM rust:1.67 as builder
WORKDIR /usr/src/resentral_server
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y wget tar firefox-esr && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/resentral_server /usr/local/bin/resentral_server

RUN wget https://github.com/mozilla/geckodriver/releases/download/v0.32.1/geckodriver-v0.32.1-linux64.tar.gz
RUN tar -xvzf geckodriver*
RUN chmod +x geckodriver
RUN mv geckodriver /usr/local/bin/geckodriver

ENV PORT=3000
EXPOSE 3000

COPY startup.sh .
RUN chmod +x startup.sh

CMD ["/bin/sh", "-c", "./startup.sh"]