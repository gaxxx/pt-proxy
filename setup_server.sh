echo $PASS
if [ -n "$PASS" ]; then 
	sed -i "s/example/$PASS/g" ss-server.json
fi

mv ss-server.json /etc/shadowsocks-libev/config.json
systemctl restart shadowsocks-libev

set -e

if timeout --preserve-status 3s cargo run server.json; then
  echo "Command succeeded"
else
  echo "Command failed with exit code $?"
fi
