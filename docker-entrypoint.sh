#!/bin/bash

if [ -z "$DATABASE_URL" ]; then
  service postgresql start
fi

export DOSEID_HOST=0.0.0.0

/usr/local/bin/doseid
