#!/bin/bash
### This is NOT used for the install, this is here for documentation and ease of reading ONLY. 
### This file is not used by anything in the json
apt update
apt-get install -y curl jq
useradd -ms /bin/bash container
chown container:nogroup /home/container
cd /home/container
DOWNLOAD_URL=$(curl -sL https://api.github.com/repos/TES3MP/openmw-tes3mp/releases/tags/0.7.0-alpha | jq -r ".assets[] | select(.name | contains(\"tes3mp-server-GNU+Linux-x86_64\")) | .browser_download_url")
curl -O -J -L $DOWNLOAD_URL
tar -xzf *.tar.gz
rm *.tar.gz
cp -r TES3MP-server/* /mnt/server/