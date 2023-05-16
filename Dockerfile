# base image
FROM ubuntu:jammy as base
RUN apt-get update
RUN apt-get install -y shadowsocks-libev
RUN apt-get install -y obfs4proxy
RUN apt-get install -y cargo
WORKDIR /app
ADD Cargo.toml .
ADD Cargo.lock .
ADD src src
RUN ls -l 
RUN env
RUN cargo build --release
RUN cp -rv target/release/pt-proxy /usr/bin/
RUN rm -rf *

# server image
FROM base as server
RUN env
WORKDIR /app
ADD ss-server.json .
ADD setup_server.sh .
ADD server.json .
EXPOSE 8020
# generate tor token in building process
ARG PASS
RUN echo $PASS
RUN bash setup_server.sh
CMD ["/bin/bash", "-c", "ss-server -c /etc/shadowsocks-libev/config.json & echo 'ss-server start' && pt-proxy server.json"]


# client image
FROM base as client
WORKDIR /app
COPY client.json .
COPY setup_client.sh .
COPY --from=server /app/obfs4_bridgeline.txt bridge.txt
ARG ADDR="127.0.0.1:8020"
RUN echo $ADDR
RUN bash setup_client.sh
CMD ["pt-proxy", "client.json"]

