# The image the remote executors run roundhouse's suites and lanes in: the
# gems the unit job installs, the Rust toolchain for emitted Rust, and Node,
# the sqlite3 CLI and Chromium's system libraries for the archive smoke.
FROM ruby:3.4
RUN gem install rails-html-sanitizer sqlite3 --no-document
ENV RUSTUP_HOME=/usr/local/rustup CARGO_HOME=/usr/local/cargo
RUN curl -sSf https://sh.rustup.rs | sh -s -- -y -q --profile minimal --default-toolchain 1.98.1 \
 && ln -s /usr/local/cargo/bin/* /usr/local/bin/
RUN curl -fsSL https://nodejs.org/dist/v20.19.5/node-v20.19.5-linux-x64.tar.xz | tar -xJ -C /usr/local --strip-components=1 \
 && apt-get update -qq && apt-get install -y -qq sqlite3 python3 >/dev/null \
 && npx -y playwright@1.56.1 install-deps chromium >/dev/null \
 && rm -rf /var/lib/apt/lists/*
LABEL org.opencontainers.image.source=https://github.com/dai199/roundhouse
