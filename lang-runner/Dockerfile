FROM rust:1.90-bullseye AS rust-build

WORKDIR /usr/src/myapp
COPY . .

RUN ls
RUN cargo build --package lang-runner --release

FROM ubuntu:25.04

# Install asdf dependencies
RUN apt-get update -y && apt-get install -y \
  curl \
  git \
  bubblewrap \
  # Python Dependencies
  gcc \
  build-essential \
  zlib1g-dev \
  libbz2-dev \
  libncurses5-dev \
  libffi-dev \
  libreadline-dev \
  libssl-dev \
  zip \
  unzip \
  # Vyxal and Kotlin Dependencies
  openjdk-17-jdk

COPY --from=denoland/deno:bin-2.5.6 /deno /usr/local/bin/deno

RUN adduser byte_heist

USER byte_heist

# Install asdf
RUN curl -L https://github.com/asdf-vm/asdf/releases/download/v0.16.4/asdf-v0.16.4-linux-amd64.tar.gz -o /home/byte_heist/asdf.tar.gz
RUN ls
RUN mkdir -p ~/.bin
RUN tar -xzf /home/byte_heist/asdf.tar.gz -C ~/.bin

# Add asdf to PATH, so it can be run in this Dockerfile
ENV PATH="$PATH:/home/byte_heist/.bin"

USER root

# Java tries to include a file in the users home directory which often fails. Disabling this functionality allows it to run more stabily.
RUN sed -i '/policy.url.2=/d' ./usr/lib/jvm/java-17-openjdk-amd64/conf/security/java.security

USER byte_heist

# Add asdf shims to PATH, so installed executables can be run in this Dockerfile
ENV PATH=$PATH:/home/byte_heist/.asdf/shims

COPY --from=rust-build /usr/src/myapp/target/release/lang-runner /lang-runner
COPY ./scripts /scripts

ENTRYPOINT ["/lang-runner"]
