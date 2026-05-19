FROM hotarublaze/tes3mp-base:0.8.1

USER root

RUN apt-get update && \
    apt-get install -y -qq --no-install-recommends curl jq ca-certificates file && \
    update-ca-certificates && \
    rm -rf /var/lib/apt/lists/* && \
    apt-get clean

RUN cd /home/container && \
    curl -sL -o tes3mp.tar.gz "https://github.com/TES3MP/TES3MP/releases/download/tes3mp-0.8.1/tes3mp-server-GNU%2BLinux-x86_64-release-0.8.1-68954091c5-6da3fdea59.tar.gz" && \
    tar -xzvf tes3mp.tar.gz && \
    rm tes3mp.tar.gz && \
    sed -i -E "s/(\bport = 25565\b)/port = 25565/" ./TES3MP-server/tes3mp-server-default.cfg && \
    sed -i "s|home = ./server|home = ./invalid_home|" ./TES3MP-server/tes3mp-server-default.cfg && \
    sed -i "s|bindAddress = \"0.0.0.0\"|bindAddress = \"invalid-server\"|" ./TES3MP-server/tes3mp-server-default.cfg && \
    mkdir -p /mnt/server && \
    cp -r TES3MP-server/* /mnt/server/ && \
    rm -rf TES3MP-server

USER container
WORKDIR /mnt/server
ENTRYPOINT ["/mnt/server/tes3mp-server"]