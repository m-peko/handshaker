FROM ubuntu:latest

LABEL maintainer="Marin Peko <marinpeko5@gmail.com>"

ENV DEPENDENCIES \
        autoconf \
        automake \
        build-essential \
        clang \
        libboost-all-dev \
        pkg-config \
        wget \
        unzip

RUN apt update && apt install -y $DEPENDENCIES

RUN wget https://github.com/bitcoin/bitcoin/archive/refs/heads/master.zip \
    && unzip master.zip \
    && rm -rf master.zip

RUN cd bitcoin-master \
    && ./autogen.sh \
    && ./configure --disable-maintainer-mode --disable-wallet --disable-tests --disable-bench --with-gui=no \
    && make -j 6 \
    && cd ..

COPY scripts/entrypoint.sh /usr/local/bin/entrypoint.sh

ENV BTC_ADDRESS=0.0.0.0 \
    BTC_CHAIN=regtest \
    BTC_TMP_DIR=/tmp/bitcoin-data \
    BTC_DAEMON_PATH=/bitcoin-master/src/bitcoind

ENTRYPOINT ["entrypoint.sh"]
CMD ["run"]
