#!/usr/bin/env sh

# Check if a custom tag has been defined, else `hjblog_rs_image`
APP_TAG="${HJBLOG_IMG:=hjblog_rs_image}"
# Check if a custom tag has been defined, else `rust_docker_skeleton`
CONTAINER_NAME="${HJBLOG_CONTAINER:=hjblog_rs}"

# Use this command after launching `build_image.sh`
docker container run -p 5000:5000 --name ${CONTAINER_NAME} ${APP_TAG}
