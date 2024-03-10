#!/bin/bash

NAME="${CONTAINER_NAME:=dev-postgres-erp}"
DB_USER="${POSTGRES_USER:=postgres}"
DB_PASSWORD="${POSTGRES_PASSWORD:=postgrespw}"
DB_NAME="${POSTGRES_DB:=erp}"
DB_PORT="${POSTGRES_PORT:=5432}"
DB_HOST="${POSTGRES_HOST:=localhost}"

export PGPASSWORD="${DB_PASSWORD}"
docker exec -it "${NAME}" psql -U "${DB_USER}" -h "${DB_HOST}" -p "${DB_PORT}" -d "${DB_NAME}"
