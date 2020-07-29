# Start with a base image containing Java runtime
FROM amazoncorretto:8u262

# Corretto is a binary distribution of the Open Java Development Kit (OpenJDK)
#  with long-term support from Amazon. Corretto is certified using the Java Technical Compatibility Kit (TCK)
# See https://hub.docker.com/_/amazoncorretto

# NB: Il seguente script e' ispirato a 
# https://hub.docker.com/r/rmohr/activemq/dockerfile

ENV TZ Europe/Rome

ENV ACTIVEMQ_VERSION 5.15.9
ENV ACTIVEMQ apache-activemq-$ACTIVEMQ_VERSION
ENV ACTIVEMQ_TCP=61616 ACTIVEMQ_AMQP=5672 ACTIVEMQ_STOMP=61613 ACTIVEMQ_MQTT=1883 ACTIVEMQ_WS=61614 ACTIVEMQ_UI=8161
ENV SHA512_VAL=35cae4258e38e47f9f81e785f547afc457fc331d2177bfc2391277ce24123be1196f10c670b61e30b43b7ab0db0628f3ff33f08660f235b7796d59ba922d444f

ENV ACTIVEMQ_HOME /opt/activemq

# GG curl is included, tar is not
# GG shadow utils is needed to get adduser 
RUN yum install -y tar shadow-utils.x86_64 gzip

RUN set -x && \
    mkdir -p /opt && \    
    curl https://archive.apache.org/dist/activemq/$ACTIVEMQ_VERSION/$ACTIVEMQ-bin.tar.gz -o $ACTIVEMQ-bin.tar.gz

# Validate checksum
RUN if [ "$SHA512_VAL" != "$(sha512sum $ACTIVEMQ-bin.tar.gz | awk '{print($1)}')" ];\
    then \
        echo "sha512 values doesn't match! exiting."  && \
        exit 1; \
    fi;

RUN tar xzf $ACTIVEMQ-bin.tar.gz -C  /opt && \
    ln -s /opt/$ACTIVEMQ $ACTIVEMQ_HOME 
RUN adduser -s /bin/bash --user-group  --home-dir  $ACTIVEMQ_HOME activemq
RUN chown -R activemq:activemq /opt/$ACTIVEMQ && \
    chown -h activemq:activemq $ACTIVEMQ_HOME

# GG Missed ps command may be needed by scripts
RUN yum install -y procps


# GG Remove build dep
#apk del build-dependencies && \
# rm -rf /var/cache/apk/*

USER activemq

# GG copy our config
ADD $ACTIVEMQ_VERSION-conf/* $ACTIVEMQ_HOME/conf/

WORKDIR $ACTIVEMQ_HOME
EXPOSE $ACTIVEMQ_TCP $ACTIVEMQ_AMQP $ACTIVEMQ_STOMP $ACTIVEMQ_MQTT $ACTIVEMQ_WS $ACTIVEMQ_UI

# GG Extra strong defaults for low latency and max 1 GB of RAM
ENV ACTIVEMQ_OPTS=" -noverify -verbose:gc -Xms1g -Xms180m -XX:+UseG1GC -XX:MaxGCPauseMillis=200 -XX:InitiatingHeapOccupancyPercent=60" 
CMD ["/bin/sh", "-c", "bin/activemq console"]

# ENTRYPOINT ["java","-Xms1024m","-Dvaadin.productionMode","-Djava.security.egd=file:/dev/./urandom","-Dspring.profiles.active=default,${SUSE_ENV}","-noverify","-server","-verbose:gc","-XX:+UseG1GC","-XX:MaxGCPauseMillis=200","-XX:InitiatingHeapOccupancyPercent=60","-jar","/suse-graduatorie-app.jar"]