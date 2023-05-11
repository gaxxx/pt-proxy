# base image
FROM ubuntu:20.04 as base
RUN apt-get update
RUN apt-get install -y shadowsocks-libev
RUN apt-get install -y obfs4proxy
RUN apt-get install -y cargo
WORKDIR /app
Add . .
RUN env
RUN cargo build

# server image
FROM base as server
RUN env
WORKDIR /app
EXPOSE 8020
# generate tor token in building process
ARG PASS
RUN echo $PASS
RUN bash setup_server.sh
CMD ["cargo", "run", "server.json"]


# client image
FROM base as client
WORKDIR /app
COPY --from=server /app/obfs4_bridgeline.txt bridge.txt
ARG ADDR="127.0.0.1:8020"
RUN echo $ADDR
RUN bash setup_client.sh
CMD ["cargo", "run", "client.json"]

