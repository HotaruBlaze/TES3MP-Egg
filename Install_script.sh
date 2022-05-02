#!/bin/bash
### This is NOT used for the install, this is here for documentation and ease of reading ONLY. 
### This file is not used by anything in the json
apt update
apt-get install -y curl jq
useradd -ms /bin/bash container
chown container:nogroup /home/container
cd /home/container
DOWNLOAD_URL=$(curl -sL https://api.github.com/repos/TES3MP/openmw-tes3mp/releases/tags/tes3mp-0.8.1 | jq -r ".assets[] | select(.name | contains(\"tes3mp-server-GNU+Linux-x86_64\")) | .browser_download_url")
curl -O -J -L $DOWNLOAD_URL
tar -xzvf *.tar.gz
rm *.tar.gz
sed -i -E "s/(\bport = 25565\b)/port = $SERVER_PORT/" ./TES3MP-server/tes3mp-server-default.cfg
cp -r TES3MP-server/* /mnt/server/