PASS=$(cat bridge.txt | grep "Bridge obfs4" | awk -F ' ' '{print $(NF-1)";"$NF}')
if [ -n "$ADDR" ]; then \
	sed -i "s/example.com:8020/$ADDR/g" client.json
fi
echo $PASS
sed -i "s#cert=HXePOA8phvqECZs6E33JimPWP9bY+bz9N+D9ehL2uyxjTVaR7sPE5op57doXUCNOyQV/LQ;iat-mode=0#$PASS#g" client.json
cat client.json
