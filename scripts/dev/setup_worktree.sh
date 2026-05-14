#!/usr/bin/env bash

mise trust
mise exec -- dotagents deploy < /dev/null
bun i --cwd tests/e2e
