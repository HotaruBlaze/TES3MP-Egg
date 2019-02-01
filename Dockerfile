FROM debian:8-slim

LABEL Author="MrFlutters <https://github.com/MrFlutters>"

ENV USER=container HOME=/home/container

RUN apt-get update && \
    apt-get install -y -qq \
    curl \
    libgl1-mesa-swx11 \
    libluajit-5.1-2 \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

RUN useradd -u 500 -ms /bin/bash ${USER} \
    && chown ${USER}:nogroup ${HOME} \
    && cd ${HOME}

USER ${USER}

WORKDIR ${HOME}

COPY ./entrypoint.sh /entrypoint.sh
ENTRYPOINT ["/bin/bash", "/entrypoint.sh" ]