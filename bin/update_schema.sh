#!/usr/bin/env bash
set -ex

SCHEMA_URL="https://cebelca-gateway.pinkstack.com/schema.graphql"
echo "Updating Schema from Čebelca Gateway from $SCHEMA_URL"
curl -sS $SCHEMA_URL > graphql/schema.graphql
