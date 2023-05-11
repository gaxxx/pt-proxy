if [-n "$PASS"]; then \
	sed -i "s/example/$PASS/g" ss-server.json
fi
mv ss-server.json /etc/shadowsocks-libev/config.json
systemd restart shadowsocks-libev
cargo run
