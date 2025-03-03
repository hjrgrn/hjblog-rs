#!/usr/bin/env bash

# Check if a custom network bridge has been set, otherwise default to `hjblog_bridge`
NETWORK_BRIDGE="${HJBLOG_BRIDGE:=hjblog_bridge}"

docker network create ${NETWORK_BRIDGE}
