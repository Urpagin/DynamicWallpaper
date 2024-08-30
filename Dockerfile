FROM rust:latest

# Set the working directory to an absolute path
WORKDIR /usr/src/app

COPY . .

WORKDIR /usr/src/app/server

RUN cargo build --release

CMD ["../target/release/server"]
