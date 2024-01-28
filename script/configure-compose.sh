#!/bin/bash

set -e

echo "
   ___  ____  ____________
  / _ \/ __ \/ __/ __/  _/
 / // / /_/ /\ \/ _/_/ /
/____/\____/___/___/___/
"
if ! [ -f .env ]; then
    cp .env.template .env

    JWT_SECRET=$(openssl rand -hex 8)
    NEW_POSTGRES_PASSWORD=$(openssl rand -hex 8)

    sed -i "s/<replace_with_secure_jwt_secret>/$JWT_SECRET/" .env
    sed -i "s/<replace_with_secure_postgres_password>/$NEW_POSTGRES_PASSWORD/" .env
    sed -i 's/127.0.0.1:5432/postgres:5432/' .env
    echo ".env file created and populated"
    echo
    echo "Run docker compose -f docker-compose.hobby.yaml up"
else
    echo "Run docker compose -f docker-compose.hobby.yaml up"
fi
echo
echo "Stuck? Join our Discord https://discord.com/invite/BP5aUkhcAh"
