#!/bin/bash

if ! [ -d yas-client ]; then
    mkdir yas-client
    cd yas-client || exit
    wget "https://linked.yashandb.com/resource/yashandb-client-23.1.1.100-linux-$(uname -m).tar.gz" -O yas-client.tar.gz
    tar xzf yas-client.tar.gz || exit 1
    cd ..
    
    exit
fi

echo "You have downloaded yas client."