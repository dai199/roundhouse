"""Totals a Bazel JSON execution log by mnemonic and runner: where a run's time went."""
import collections
import json
import sys

def spawns(path):
    dec = json.JSONDecoder()
    buf = open(path).read()
    i = 0
    while i < len(buf):
        while i < len(buf) and buf[i].isspace():
            i += 1
        if i >= len(buf):
            break
        obj, i = dec.raw_decode(buf, i)
        yield obj

def seconds(d):
    return float(d.rstrip("s")) if d else 0.0

total = collections.defaultdict(lambda: [0, 0.0])
for s in spawns(sys.argv[1]):
    runner = s.get("runner", "?")
    if s.get("remoteCacheHit") or runner == "remote cache hit":
        runner = "cache hit"
    t = seconds(s.get("metrics", {}).get("totalTime") or s.get("walltime"))
    key = (s.get("mnemonic", "?"), runner)
    total[key][0] += 1
    total[key][1] += t

print("| mnemonic | runner | count | seconds |\n|---|---|---|---|")
for (m, r), (n, t) in sorted(total.items(), key=lambda kv: -kv[1][1])[:25]:
    print(f"| {m} | {r} | {n} | {t:.0f} |")
