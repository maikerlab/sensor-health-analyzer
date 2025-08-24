#!/bin/sh
# 1. Run all pending migrations
# shellcheck disable=SC2164
cd /app
export DATABASE_URL="$IOT_DATABASE_URL"
sqlx database create
sqlx migrate run

# 2. Run Dispatcher
/app/dispatcher
