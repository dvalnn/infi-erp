#!/bin/bash
#
# This script is used to initialize a postgres instance for development.
# For deployment, the docker-compose file should be used.

# enable debugging prints
set -x
# exit on error
set -eo pipefail

# Check if psql is installed
if ! [ -x "$(command -v psql)" ]; then
	echo 'Error: psql is not installed.' >&2
	exit 1
fi

# Check if sqlx-cli is installed
if ! [ -x "$(command -v sqlx)" ]; then
	echo 'Error: sqlx-cli is not installed.' >&2
	echo 'Please install sqlx-cli by running: '
	echo "cargo install --version='~0.7' sqlx-cli --no-default-features --features postgres,rustls"
	exit 1
fi

# Check for database variables otherwise use defaults
DB_USER="${POSTGRES_USER:=postgres}"
DB_PASSWORD="${POSTGRES_PASSWORD:=postgrespw}"
DB_NAME="${POSTGRES_DB:=erp}"
DB_PORT="${POSTGRES_PORT:=5432}"
DB_HOST="${POSTGRES_HOST:=localhost}"

if [[ -z "${SKIP_DOCKER}" ]]; then
	docker run \
		--name dev-postgres-erp \
		-e POSTGRES_USER=${DB_USER} \
		-e POSTGRES_PASSWORD=${DB_PASSWORD} \
		-e POSTGRES_DB=${DB_NAME} \
		-p "${DB_PORT}":5432 \
		-d postgres:16-alpine \
		postgres -N 1000
# ^ Increased maximum number of connections for testing purposes
fi

export PGPASSWORD="${DB_PASSWORD}"
until psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
	>&2 echo "Postgres is unavailable - sleeping"
	sleep 1
done

>&2 echo "Postgres is up and running on ${DB_HOST}:${DB_PORT}!"

DATABASE_URL=postgresql://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}
export DATABASE_URL
sqlx database create
sqlx migrate run

>&2 echo "Postgres has been migrated, ready to go!"
