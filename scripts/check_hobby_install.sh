#!/bin/bash

set -e

if ! [ -f .env ]; then
    echo "The .env file is missing. Run make install-hobby"
    exit 1
fi
