#!/bin/bash
docker exec -it erp-postgres-1 psql -h localhost -p 5432 -U postgres -W
