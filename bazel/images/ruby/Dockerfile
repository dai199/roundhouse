# The image the Ruby-driving test suites run in on a remote executor: the
# gems the unit job installs before `cargo test`.
FROM ruby:3.4
RUN gem install rails-html-sanitizer sqlite3 --no-document
LABEL org.opencontainers.image.source=https://github.com/dai199/roundhouse
