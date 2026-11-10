#!/bin/bash
# 0 when the regressed Spinel refuses Article's reader and the earlier one does not
P='app/app/models/article.rb:1: unsupported declared return type is a typed object array'
timeout 300 "$BAD" contract.rb -c -o /dev/null > /tmp/bad.$$ 2>&1 &
timeout 300 "$GOOD" contract.rb -c -o /dev/null > /tmp/good.$$ 2>&1 &
wait
r=1
grep -qF "$P" /tmp/bad.$$ && ! grep -qF "$P" /tmp/good.$$ && grep -q 'Wrote' /tmp/good.$$ && r=0
rm -f /tmp/bad.$$ /tmp/good.$$
exit $r
