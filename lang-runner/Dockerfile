FROM rust:1.81-bullseye AS rust-build

WORKDIR /usr/src/myapp
COPY . .

RUN ls
RUN cargo build --package lang-runner --release

FROM debian:12

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
  # Java, because Java is a thing some languages need.
  openjdk-17-jdk


RUN ln /usr/bin/java /usr/local/bin/java

RUN adduser yq

USER yq

# Install asdf
RUN git clone --depth 1 https://github.com/asdf-vm/asdf.git ~/.asdf

# Add asdf to PATH, so it can be run in this Dockerfile
ENV PATH="$PATH:/home/yq/.asdf/bin"

# Add asdf shims to PATH, so installed executables can be run in this Dockerfile
ENV PATH=$PATH:/home/yq/.asdf/shims

# Set JAVA_HOME
ENV JAVA_HOME="/usr/lib/jvm/java-17-openjdk-amd64"
ENV PATH="$JAVA_HOME/bin:$PATH"

COPY --from=rust-build /usr/src/myapp/target/release/lang-runner /lang-runner
COPY ./scripts /scripts

ENTRYPOINT /lang-runner
