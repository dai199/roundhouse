"""The remote executors' CI image; bump the tag whenever bazel/images/ruby/Dockerfile changes."""

# Not a moving tag: an executor caches an image by name and keeps running the stale one.
CI_IMAGE = "docker://ghcr.io/dai199/roundhouse-ci-ruby:3.4-v2"
