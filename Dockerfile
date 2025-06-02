# syntax=docker/dockerfile:1

# This Dockerfile's original author is unknown: maybe Casey
# Bailey or Bastian Gruber. Bart Massey adapted it for this
# project.

ARG RUST_VERSION=1.87
ARG APP_NAME=kk2

################################################################################
# Create a stage for building the application.

#FROM rust:${RUST_VERSION}-alpine AS build
FROM rust:${RUST_VERSION} AS build
ARG APP_NAME
WORKDIR /app

ENV DATABASE_URL=sqlite:db/knock-knock.db

# Install host build dependencies.
RUN apt-get install git curl

# Build the application.
# Leverage a cache mount to /usr/local/cargo/registry/
# for downloaded dependencies, a cache mount to /usr/local/cargo/git/db
# for git repository dependencies, and a cache mount to /app/target/ for
# compiled dependencies which will speed up subsequent builds.
# Leverage a bind mount to the src directory to avoid having to copy the
# source code into the container. Once built, copy the executable to an
# output directory before the cache mounted /app/target is unmounted.
RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=build.rs,target=build.rs \
    --mount=type=bind,source=askama.toml,target=askama.toml \
    --mount=type=bind,source=assets,target=assets \
    --mount=type=bind,source=migrations,target=migrations \
    --mount=type=bind,source=db,target=db \
    --mount=type=bind,source=.sqlx,target=.sqlx \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
cargo build --release && \
cp ./target/release/$APP_NAME /bin/server

################################################################################
# Create a new stage for running the application that contains the minimal
# runtime dependencies for the application. This often uses a different base
# image from the build stage where the necessary files are copied from the build
# stage.
#
# The example below uses the alpine image as the foundation for running the app.
# By specifying the "3.18" tag, it will use version 3.18 of alpine. If
# reproducability is important, consider using a digest
# (e.g., alpine@sha256:664888ac9cfd28068e062c991ebcff4b4c7307dc8dd4df9e728bedde5c449d91).
#FROM alpine:latest AS final
FROM rust:${RUST_VERSION} AS final

# Create a non-privileged user that the app will run under.
# See https://docs.docker.com/go/dockerfile-user-best-practices/
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser

# Copy the executable from the "build" stage.
COPY --from=build /bin/server /bin/

COPY --chown=appuser:appuser ./assets ./assets
COPY --chown=appuser:appuser ./migrations ./migrations
COPY --chown=appuser:appuser ./db ./db
COPY --chown=appuser:appuser ./secrets ./secrets

# Remember to expose the port that the application listens on.
# This is the port it is supposed to listen on.
EXPOSE 3000

# What the container should run when it is started.
CMD ["/bin/server", "-i", "0.0.0.0"]
