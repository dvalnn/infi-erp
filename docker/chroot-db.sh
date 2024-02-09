#!/bin/bash

# attach to a shell session in the running postgres container
docker exec -it infi-postgresdb psql -d infi-postgres -U admin
