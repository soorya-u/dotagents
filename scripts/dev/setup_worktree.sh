#!/usr/bin/env bash

mise trust
touch .dotagents/local.config.toml # a hack to make dotagents run. will be fixed in next update
mise exec -- dotagents deploy < /dev/null
bun i --cwd tests/e2e
