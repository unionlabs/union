# we need python2 support, which was dropped after buster:
FROM debian:buster

RUN echo 'debconf debconf/frontend select Noninteractive' | debconf-set-selections
RUN apt-get update
RUN apt-get install -y apt-utils

# Install and configure locale `en_US.UTF-8`
RUN apt-get install -y locales && \
    sed -i -e "s/# $en_US.*/en_US.UTF-8 UTF-8/" /etc/locale.gen && \
    dpkg-reconfigure --frontend=noninteractive locales && \
    update-locale LANG=en_US.UTF-8
ENV LANG=en_US.UTF-8

RUN apt-get update
RUN apt-get install -y git python2 python-pip g++ cmake python-ply python-tk tix pkg-config libssl-dev python-setuptools

# create a user:
RUN useradd -ms /bin/bash user
USER user
WORKDIR /home/user

RUN git clone --recurse-submodules https://github.com/kenmcmil/ivy.git
WORKDIR /home/user/ivy/
RUN git checkout 271ee38980699115508eb90a0dd01deeb750a94b

RUN python2.7 build_submodules.py
RUN mkdir -p "/home/user/python/lib/python2.7/site-packages"
ENV PYTHONPATH="/home/user/python/lib/python2.7/site-packages"
# need to install pyparsing manually because otherwise wrong version found
RUN pip install pyparsing
RUN python2.7 setup.py install --prefix="/home/user/python/"
ENV PATH=$PATH:"/home/user/python/bin/"
WORKDIR /home/user/tendermint-proof/

ENTRYPOINT ["/home/user/tendermint-proof/check_proofs.sh"]

