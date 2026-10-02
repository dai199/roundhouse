"""The remote executors' CI image; bump the tag whenever bazel/images/all/Dockerfile changes."""

# Not a moving tag: an executor caches an image by name and keeps running the stale one.
CI_IMAGE = "docker://ghcr.io/dai199/roundhouse-ci-all:v1"

# once-campfire at the commit ci.yml pins.
CAMPFIRE_SHA = "90b330024dec3e757c79b6a7e6568f93da8e3148"
