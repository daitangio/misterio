FROM alpine:3.12

RUN apk update \
 && apk upgrade \
 && apk add --no-cache \
            rsync \
            openssh-client \
            ca-certificates \
 && update-ca-certificates \
 && rm -rf /var/cache/apk/*
RUN uname -a
COPY backup-me.sh /root/
CMD sleep 3600