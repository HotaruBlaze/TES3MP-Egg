FROM hotarublaze/tes3mp-base:0.8.1

COPY ./entrypoint.sh /entrypoint.sh
ENTRYPOINT ["/bin/bash", "/entrypoint.sh"]