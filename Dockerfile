FROM arm64v8/ubuntu:22.04

LABEL Author="hotarublaze <https://github.com/hotarublaze>"

ENV USER=container HOME=/home/container

RUN dpkg --add-architecture armhf \
    && apt-get update && apt-get upgrade -y \
    && apt-get update \
    && apt-get install -y -qq \
        curl \
        libgl1-mesa-glx \
        libluajit-5.1:armhf \
        zlib1g:armhf \
        gcc-arm-linux-gnueabihf \
        libstdc++6:armhf \
        libbz2-1.0:armhf \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

RUN useradd -u 500 -ms /bin/bash ${USER} \
    && chown ${USER}:nogroup ${HOME} \
    && cd ${HOME}

USER ${USER}

WORKDIR ${HOME}

COPY ./entrypoint.sh /entrypoint.sh
ENTRYPOINT ["/bin/bash", "/entrypoint.sh"]