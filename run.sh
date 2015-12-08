#!/bin/bash

CONFIG=$(readlink -f "$1")
shift

PORT="$1"
if [ -z "$PORT" ]
then
	PORT=8080
fi
shift

ARGS="$@"

docker run -d -p $PORT:8080 -v "$CONFIG:/config.toml" --name farmview $ARGS farmview
