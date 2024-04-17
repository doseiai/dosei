#!/bin/bash

if [ -z "$DATABASE_URL" ]; then
  service postgresql start
fi

export DOSEI_HOST=0.0.0.0

/usr/local/bin/doseid
