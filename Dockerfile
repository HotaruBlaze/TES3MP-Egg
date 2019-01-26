FROM debian:8-slim

LABEL Author="MrFlutters <https://github.com/MrFlutters>"

ENV  USER=container HOME=/home/container

RUN apt-get update && \
    apt-get install -y -qq \
    curl \
    libgl1-mesa-swx11 \
    libqt5widgets5 \
    libasound2 \
    libpulse0 \
    libxss1 \
    libxrandr2 \
    libluajit-5.1-2 \
    libxxf86vm1  \
    libwayland-cursor0 \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

RUN useradd -ms /bin/bash ${USER} \
    && chown ${USER}:nogroup ${HOME} \
    && cd ${HOME}

USER container

WORKDIR /home/container

COPY ./entrypoint.sh /entrypoint.sh
ENTRYPOINT ["/bin/bash", "/entrypoint.sh" ]