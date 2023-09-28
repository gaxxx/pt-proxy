# How to use this with obfs4proxy

This tools is used to secure your traffic with the lastest obfuscation protocol.

jump to Docker section if you don't bother to check the detailed bullshit.


## Server side
1. buy a vpn server, better to use Ubuntu, setup shadowsocks & obfsproxy4,
2. setup server.json, here are some tips
	* local: your local ss port
	* server: your obfuscated port for your client
3. cargo run server.json
4. check out the output,there should be some output like belowing. That is your PASSWORD.
```
	cert=HXePOA8phvqECZs6E33JimPWP9bY+bz9N+D9ehL2uyxjTVaR7sPE56666doXUCNOyQV/LQ;iat-mode
```


## Client side
1. prepare your linux box in respberry pi or nas, better to use Ubuntun and have shadowsocks & obfs4proxy installed.
2. setup client.json
	* local: your local ss port
	* server: your vpn ip and port in "Server side" section
	* ptargs: your PASSWORD in "Server side" section
3. cargo run client.json

Then you can enjoy your pornhub. You could also integrate these into your systemd or supervisord to avoid some dup work.


## Docker
1. clone this repo
2. register an account in dockerhub, let's say it's gaxxx
```
docker login
```
3. run following cmd, the PASS is the shadowsocks password, the ADDR is your vpn addr:port, just use test.com:8020 for example
```
docker build -t gaxxx/pt-proxy-client --target client --build-arg PASS=hello --build-arg ADDR=test.com .
docker build -t gaxxx/pt-proxy-server --target server --build-arg PASS=hello --build-arg ADDR=test.com .

docker push gaxxx/pt-proxy-client
docker push gaxxx/pt-proxy-server
```
in this way we've got 2 dockers for server & client and register them in dockerhub

4. in your vpn server run following 
```
docker run -p 8020:8020 gaxxx/pt-proxy-server
```
5. in your local service run following
```
docker run -p 8020:8020 gaxxx/pt-proxy-client
```
6. use [shadowsocks-ng](https://github.com/shadowsocks/ShadowsocksX-NG) to add your own ss profiles

btw: put your shadowsocks passwd to the Password field
<img width="557" alt="image" src="https://github.com/gaxxx/pt-proxy/assets/471881/9bf4ab4f-a4ab-481f-be77-fc2351223766">

## FAQ
N/A





