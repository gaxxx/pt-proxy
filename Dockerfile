FROM ubuntu:jammy
RUN apt-get install shadowsocks-libev
RUN apt-get install obfs4proxy
RUN cargo run
ENV PASS=example
ADD .
EXPOSE 8020
CMD ["sh", "run.sh"]
