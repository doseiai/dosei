#!/bin/bash

source .env

# Prompt for the GitHub token
echo "Please enter your GitHub access token:"
read -s ACCESS_TOKEN

curl -u "$GITHUB_CLIENT_ID":"$GITHUB_CLIENT_SECRET" \
     -H "Accept: application/vnd.github.v3+json" \
     https://api.github.com/applications/"$GITHUB_CLIENT_ID"/token \
     -d "{\"access_token\":\"$ACCESS_TOKEN\"}"
