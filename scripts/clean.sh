#!/usr/bin/env sh

APP_TAG="${HJBLOG_IMG:=hjblog_rs_image}"
CONTAINER_NAME="${HJBLOG_CONTAINER:=hjblog_rs}"
NETWORK_BRIDGE="${HJBLOG_BRIDGE:=hjblog_bridge}"
DB_INSTANCE="${HJBLOG_POSTGRES_INSTANCE:=hjblog_postgres_instance}"

docker container rm -f ${CONTAINER_NAME}
docker container stop ${DB_INSTANCE}
docker container rm ${DB_INSTANCE}
docker image rm ${APP_TAG}
docker image prune -f
docker network rm ${NETWORK_BRIDGE}
