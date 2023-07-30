#!/bin/sh

cd /app
git clone --no-checkout $GIT_REPOSITORY app
cd app
git sparse-checkout set migrations
git checkout

diesel migration run
