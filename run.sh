#!/bin/bash

CONFIG=$(readlink -f "$1")
CONFIG_ARG="$CONFIG:/config.toml"
PORT="$2"
if [ -z "$PORT" ]
then
	PORT=8080
fi

docker run --rm -p $PORT -v "$CONFIG_ARG" --name farmview farmview
