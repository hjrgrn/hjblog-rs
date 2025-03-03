#!/usr/bin/env sh

# Check if a custom tag has been defined, else `hjblog_rs_image`
APP_TAG="${HJBLOG_IMG:=hjblog_rs_image}"
# Check if a custom tag has been defined, else `hjblog_rs`
CONTAINER_NAME="${HJBLOG_CONTAINER:=hjblog_rs}"
# Check if a custom tag has been defined, else `hjblog_bridge`
NETWORK_BRIDGE="${HJBLOG_BRIDGE:=hjblog_bridge}"

# Use this command after launching `build_image.sh`
docker container run -p 5000:5000 --network ${NETWORK_BRIDGE} --name ${CONTAINER_NAME} ${APP_TAG}
