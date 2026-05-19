FROM debian:bookworm-slim

LABEL Author="hotarublaze <https://github.com/hotarublaze>"

ENV USER=container HOME=/home/container

RUN apt-get update && \
    apt-get install -y -qq --no-install-recommends \
    curl \
    libgl1-mesa-glx \
    libluajit-5.1-2 \
    libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

RUN useradd -u 500 -ms /bin/bash ${USER} \
    && chown ${USER}:nogroup ${HOME} \
    && mkdir -p ${HOME}/logs \
    && cd ${HOME}

USER ${USER}

WORKDIR ${HOME}