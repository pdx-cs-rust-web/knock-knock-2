# syntax=docker/dockerfile:1

# This Dockerfile's original author is unknown: maybe Casey
# Bailey or Bastian Gruber. Bart Massey adapted it for this
# project.

ARG RUST_VERSION=1.87

################################################################################
# Create a stage for building the application.

FROM rust:${RUST_VERSION} AS build
WORKDIR /build

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
    cp target/release/kk2 /bin/kk2

# Create a non-privileged user that the app will run under.
# See https://docs.docker.com/go/dockerfile-user-best-practices/
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --shell "/sbin/nologin" \
    --uid "${UID}" \
    appuser
USER appuser

WORKDIR /home/appuser

COPY --chown=appuser:appuser assets ./assets
COPY --chown=appuser:appuser migrations ./migrations
COPY --chown=appuser:appuser db ./db
COPY --chown=appuser:appuser secrets ./secrets

# Remember to expose the port that the application listens on
# with -p 3000:300
# This does not do that.
EXPOSE 3000

# What the container should run when it is started.
CMD ["/bin/kk2", "-i", "0.0.0.0"]
