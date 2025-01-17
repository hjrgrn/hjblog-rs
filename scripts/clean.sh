#!/usr/bin/env sh

# Check if a custom tag has been defined, else `hjblog_rs_image`
APP_TAG="${HJBLOG_IMG:=hjblog_rs_image}"
# Check if a custom tag has been defined, else `rust_docker_skeleton`
CONTAINER_NAME="${HJBLOG_CONTAINER:=hjblog_rs}"

docker container rm -f ${CONTAINER_NAME}
docker image rm ${APP_TAG}
docker image prune -f
