# The image the Ruby-driving test suites run in on a remote executor: the
# gems the unit job installs before `cargo test`, and the Rust toolchain
# the suites that compile emitted Rust call.
FROM ruby:3.4
RUN gem install rails-html-sanitizer sqlite3 --no-document
ENV RUSTUP_HOME=/usr/local/rustup CARGO_HOME=/usr/local/cargo
RUN curl -sSf https://sh.rustup.rs | sh -s -- -y -q --profile minimal --default-toolchain 1.98.1 \
 && ln -s /usr/local/cargo/bin/* /usr/local/bin/
LABEL org.opencontainers.image.source=https://github.com/dai199/roundhouse
