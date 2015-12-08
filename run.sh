#!/bin/bash

CONFIG=$(readlink -f "$1")
echo $CONFIG

shift

PORT="$1"
if [ -z "$PORT" ]
then
	PORT=8080
fi
echo $PORT

shift

ARGS="$@"
echo $ARGS

docker run --rm -p $PORT -v "$CONFIG:/config.toml" --name farmview $ARGS farmview
