FROM rust:1.85-bullseye AS rust-build

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
  # Vyxal Dependencies
  openjdk-17-jdk

RUN adduser yq

USER yq

# Install asdf
RUN git clone --depth 1 https://github.com/asdf-vm/asdf.git ~/.asdf

# Add asdf to PATH, so it can be run in this Dockerfile
ENV PATH="$PATH:/home/yq/.asdf/bin"

# Add asdf shims to PATH, so installed executables can be run in this Dockerfile
ENV PATH=$PATH:/home/yq/.asdf/shims

COPY --from=rust-build /usr/src/myapp/target/release/lang-runner /lang-runner
COPY ./scripts /scripts

ENTRYPOINT /lang-runner
