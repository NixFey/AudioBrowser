FROM rust:1.83
EXPOSE 3000
WORKDIR /usr/src/AudioBrowser
COPY . .

RUN cargo install --path .

CMD ["AudioBrowser"]