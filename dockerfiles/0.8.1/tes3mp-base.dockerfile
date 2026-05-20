ARG DEBIAN_BASE=trixie
FROM debian:${DEBIAN_BASE}-slim

LABEL Author="hotarublaze <https://github.com/hotarublaze>"

ENV USER=container HOME=/home/container

RUN apt-get update && \
    apt-get install -y -qq --no-install-recommends \
    curl \
    ca-certificates \
    libgl1 \
    libluajit-5.1-2 \
    libssl3t64 \
    p7zip-full \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

RUN useradd -u 500 -ms /bin/bash ${USER} \
    && chown ${USER}:nogroup ${HOME} \
    && mkdir -p ${HOME}/logs \
    && cd ${HOME}

USER ${USER}

WORKDIR ${HOME}