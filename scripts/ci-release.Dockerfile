ARG os_image=centos:centos7
FROM ${os_image}

RUN yum install -y wget git centos-release-scl centos-release-scl-rh
RUN yum install -y devtoolset-7-gcc devtoolset-7-gcc-g++ llvm-toolset-7.0-clang llvm-toolset-7.0-clang-libs gcc g++
RUN sh -c "$(curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs)" -- -y

ENTRYPOINT [ "bash" ]
