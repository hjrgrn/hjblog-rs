#!/usr/bin/env sh

# Check if a custom tag has been defined, else `hjblog_rs_image`
APP_TAG="${HJBLOG_IMG:=hjblog_rs_image}"

docker image build --tag ${APP_TAG} --file Dockerfile .
