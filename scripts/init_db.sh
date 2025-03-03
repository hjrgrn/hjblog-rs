#!/usr/bin/env bash

# Init Postgres database for local and test environment

set -x
set -eo pipefail

# Check is the programs we will need are installed
if ! [ -x "$(command -v psql)" ]; then
    echo >&2 "Error: psql is not installed."
    exit 1
fi
if ! [ -x "$(command -v sqlx)" ]; then
    echo >&2 "Error: sqlx is not installed."
    echo >&2 "Use:"
    echo >&2 "cargo install sqlx-cli --no-default-features --features rustls,postgres"
    echo >&2 "to install it."
    exit 1
fi

# Check if a custom informations have been set in the environment, otherwise use the defaults
DB_USER=${POSTGRES_USER:=postgres}
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
DB_NAME="${POSTGRES_DB:=hjblog_pg}"
DB_PORT="${POSTGRES_PORT:=5432}"
DB_HOST="${POSTGRES_HOST:=localhost}"
NETWORK_BRIDGE="${HJBLOG_BRIDGE:=hjblog_bridge}"
DB_INSTANCE="${HJBLOG_POSTGRES_INSTANCE:=hjblog_postgres_instance}"

# Allow to skip Docker if a dockerized Postgres database is already running
if [[ -z "${SKIP_DOCKER}" ]]; then
    docker container run \
        -e POSTGRES_USER=${DB_USER} \
        -e POSTGRES_PASSWORD=${DB_PASSWORD} \
        -e POSTGRES_DB=${DB_NAME} \
        --name ${DB_INSTANCE} \
        --network ${NETWORK_BRIDGE} \
        -p "${DB_PORT}":5432 \
        -d postgres \
        postgres -N 1000
        # ^ increase maximum number of connections for testing purpose
fi

# Keep pinging Postgres until it's ready to accept commands, sometimes the instance will
# take a bit to start; this command tries to open a connection, if it manages to do so it closes it right away,
# otherwise it retries.
export PGPASSWORD="${DB_PASSWORD}"
until psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
    >&2 echo "Postgres is still unavaible - sleeping"
    sleep 1
done

>&2 echo "Postgres is up and running on port ${DB_PORT}!"

# `sqlx` cli requires a global variable `DATABASE_URL`
# that represent a vaild postgres url.
DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}
export DATABASE_URL
sqlx database create
sqlx migrate run

>&2 echo "Postgres has been migrated, ready to go!"
