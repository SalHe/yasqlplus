#!/usr/bin/bash

# shellcheck disable=SC2164
cd ~
mkdir install_yashandb
cd install_yashandb

wget "https://linked.yashandb.com/resource/yashandb-personal-23.1.1.100-linux-$(uname -m).tar.gz" --no-check-certificate -O yashandb-personal.tar.gz
tar xzf yashandb-personal.tar.gz
./scripts/install.sh
./scripts/initDB.sh
