import sys, collections
# perf script output: blocks separated by blank lines; first line header, then frames "addr sym+off (dso)"
targets = sys.argv[1:]
total = 0
inc = collections.Counter()
below = {t: collections.Counter() for t in targets}
block = []
def flush(block):
    global total
    if not block: return
    frames = []
    for l in block:
        p = l.strip().split()
        if len(p) >= 2 and all(ch in '0123456789abcdef' for ch in p[0]):
            frames.append(p[1].split('+')[0])
    total += 1
    seen = set()
    for i, f in enumerate(frames):
        if f in below and f not in seen:
            seen.add(f); inc[f] += 1
            below[f][frames[i-1] if i > 0 else '(self)'] += 1
for line in sys.stdin:
    if line.strip() == '':
        flush(block); block = []
    else:
        block.append(line)
flush(block)
for t in targets:
    print(f"== {t}: {100*inc[t]/max(total,1):.1f}% of samples")
    for k, v in below[t].most_common(15):
        print(f"   {100*v/max(total,1):5.2f}%  {k}")
