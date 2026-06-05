#!/usr/bin/env bash

mise trust
mise exec -- dotagents deploy --ci
bun i --cwd tests/e2e
