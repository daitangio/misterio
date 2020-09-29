FROM jenkins/jenkins:2.163
# Last diana jenkins is 2.163

ENV TZ Europe/Rome
ENV LANG en_US.UTF-8

WORKDIR /var
USER root
#COPY --chown=jenkins:jenkins agos_jenkins_home.tgz .
#RUN rm -rf /var/jenkins_home/*
#RUN tar zxf agos_jenkins_home.tgz 
#RUN rm agos_jenkins_home.tgz

# GG: Add docker interop
# The idea is to expose the docker socket from the docker host, running the image via -v "/var/run/docker.sock:/var/run/docker.sock"
# We need also the docker command line, so we need some "magic" steps below
# Refer to https://docs.docker.com/install/linux/docker-ce/debian/ for explanation of curl/add-apt-repository steps
USER root

RUN apt update
RUN apt install -y software-properties-common apt-transport-https
RUN curl -fsSL https://download.docker.com/linux/debian/gpg |  apt-key add
RUN add-apt-repository \
   "deb [arch=amd64] https://download.docker.com/linux/debian \
   $(lsb_release -cs) \
   stable"
RUN apt update
RUN apt install -y docker-ce-cli
RUN groupadd docker && usermod -aG docker jenkins
# During the share the docker socket will be on root group. We need this line to get jenkins working with docker
RUN usermod -aG root jenkins


RUN curl -L "https://github.com/docker/compose/releases/download/1.22.0/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
RUN chmod a+rx /usr/local/bin/docker-compose
RUN docker-compose --version




USER jenkins
WORKDIR /var/jenkins_home
# Enable a self contained and shared git daemon (FUNNY)
USER root
COPY ./runGitAndJenkins.sh /usr/local/bin/
RUN chmod a+rx /usr/local/bin/runGitAndJenkins.sh
# runGitAndJenkins will use the jenkins user
USER root
ENTRYPOINT ["/sbin/tini", "--", "/usr/local/bin/runGitAndJenkins.sh"]
