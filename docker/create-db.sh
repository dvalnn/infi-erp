#!/bin/bash

# Start the postgres database container
# Only need to be run once per machine
# Will create a container running postgresql
# over a debian OS image
docker run --name infi-postgresdb \
	-e POSTGRES_PASSWORD=admin \
	-e POSTGRES_USER=admin \
	-e POSTGRES_DB=infi-postgres \
	-p 5432:5432 \
	postgres
