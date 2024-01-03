FROM centos:centos7

RUN yum install -y wget git centos-release-scl centos-release-scl-rh
RUN yum install -y devtoolset-7-gcc devtoolset-7-gcc-g++ llvm-toolset-7 gcc g++
RUN sh -c "$(curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs)" -- -y

COPY ./ /yasqlplus
RUN /yasqlplus/scripts/ci-release-build.sh

ENTRYPOINT [ "bash" ]
