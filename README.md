# How to use this with obfs4proxy

This tools is used to secure your traffic with the lastest obfuscation protocal.

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
WIP






