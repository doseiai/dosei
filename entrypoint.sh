#!/bin/bash

if [ -z "$DATABASE_URL" ]; then
  service postgresql start
  export DATABASE_URL=postgres://postgres@host/postgres?host=/var/run/postgresql
fi

if [ -z "$CONTAINER_REGISTRY_URL" ]; then
  export CONTAINER_REGISTRY_URL=ghcr.io
fi

/usr/local/bin/doseid --host 0.0.0.0
