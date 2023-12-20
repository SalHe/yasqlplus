#!/bin/bash

TARGET_DIR=./yasqlplus-client/yas-client
if ! [ -d $TARGET_DIR ]; then
    WORK_DIR="$(pwd)"
    mkdir $TARGET_DIR
    cd $TARGET_DIR || exit
    wget "https://linked.yashandb.com/resource/yashandb-client-23.1.1.100-linux-$(uname -m).tar.gz" --no-check-certificate -O yas-client.tar.gz
    tar xzf yas-client.tar.gz || exit 1
    cd "$WORK_DIR" || exit

    exit
fi

echo "You have downloaded yas client."
