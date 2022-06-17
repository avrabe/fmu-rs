# See here for image contents: https://github.com/microsoft/vscode-dev-containers/tree/v0.236.0/containers/rust/.devcontainer/base.Dockerfile

# [Choice] Debian OS version (use bullseye on local arm64/Apple Silicon): buster, bullseye
ARG VARIANT="buster"
FROM mcr.microsoft.com/vscode/devcontainers/rust:0-${VARIANT}

# [Optional] Uncomment this section to install additional packages.
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
     && apt-get -y install --no-install-recommends build-essential pkg-config dh-autoreconf libglib2.0-dev bison liblzma-dev libext2fs-dev libgpgme-dev libfuse-dev \
     && git clone --depth 1 --branch v2022.1 https://github.com/ostreedev/ostree.git \
     && cd ostree \
     && git submodule update --init \
     && env NOCONFIGURE=1 ./autogen.sh \
     && ./configure --without-gpgme --prefix=/usr/ \
     && make -j 20 \
     && make install