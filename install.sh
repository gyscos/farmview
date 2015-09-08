#!/bin/sh
NAME=farmview
mkdir -p "$DESTDIR/usr/bin"
mkdir -p "$DESTDIR/usr/share/$NAME"
mkdir -p "$DESTDIR/etc/$NAME"

cp -a templates "$DESTDIR/usr/share/$NAME/"
cp config.toml "$DESTDIR/etc/$NAME/"
cp $NAME "$DESTDIR/usr/bin/"
