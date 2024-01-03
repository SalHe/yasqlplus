#!/bin/bash

cat >>/etc/ld.so.conf <<EOF
/usr/local/lib
/usr/local/lib64
/usr/lib64/clang-private
EOF

cat /etc/ld.so.conf

source /opt/rh/devtoolset-7/enable
source /opt/rh/llvm-toolset-7/enable
source "$HOME/.cargo/env"

ldconfig

mkdir yasdb
cd yasdb
wget "https://linked.yashandb.com/resource/yashandb-personal-23.1.1.100-linux-$(uname -m).tar.gz" --no-check-certificate -O yashandb-personal.tar.gz
tar xzf yashandb-personal.tar.gz
cd ..

export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:$(pwd)/yasdb/lib

cd yasqlplus
./scripts/download-dependency.sh
cargo build --release
