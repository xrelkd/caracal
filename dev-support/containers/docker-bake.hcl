group "default" {
  targets = ["caracal"]
}

target "caracal" {
  dockerfile = "dev-support/containers/alpine/Containerfile"
  platforms  = ["linux/amd64"]
  target     = "caracal"
  contexts = {
    rust   = "docker-image://docker.io/library/rust:1.76.0-alpine3.19"
    alpine = "docker-image://docker.io/library/alpine:3.19"
  }
  args = {
    RUSTC_WRAPPER         = "sccache"
    AWS_ACCESS_KEY_ID     = null
    AWS_SECRET_ACCESS_KEY = null
    SCCACHE_BUCKET        = null
    SCCACHE_ENDPOINT      = null
    SCCACHE_S3_USE_SSL    = null
  }
  labels = {
    "description"                     = "Container image for Caracal"
    "image.type"                      = "final"
    "image.authors"                   = "46590321+xrelkd@users.noreply.github.com"
    "image.vendor"                    = "xrelkd"
    "image.description"               = "Caracal - File downloader written in Rust Programming Language"
    "org.opencontainers.image.source" = "https://github.com/xrelkd/caracal"
  }
}
