#!/bin/bash

cd /app
git clone --no-checkout $GIT_REPOSITORY app
cd app
git sparse-checkout set migrations
git checkout $GIT_COMMIT

diesel migration run

echo "Ran migrations successfully!"
echo "GIT_REPOSITORY: $GIT_REPOSITORY"
echo "GIT_COMMIT: $GIT_COMMIT"
echo "PGUSER: $PGUSER"
echo "PGHOST: $PGHOST"
echo "PGPORT: $PGPORT"
echo "PGDATABASE: $PGDATABASE"
exit 0
