import subprocess
import os
import re
import time
import statistics
from pathlib import Path

BENCH_TIMEOUT = 5

root_dir = Path(__file__).absolute().parent.parent
puzzles_file = root_dir / 'www' / 'index.html'
rs_dir = root_dir / 'rs'
executable = rs_dir / 'target' / 'release' / 'solvepuzzle'

# parse puzzles
with open(puzzles_file, 'r', encoding='utf-8') as fd:
    puzzles_txt = re.search(r'const PUZZLES_TXT = `([^`]*)`', fd.read(), re.MULTILINE)[1]

def parse_puzzles(txt):
    parsed = []
    sections = {}
    top_section = 0
    for line in filter(bool, txt.splitlines()):
        ident, text = re.match(r'(\s*)(.+)', line).groups()
        if text.startswith('#'):
            parsed.append({
                'section': [sec for _, sec in sorted((v, sec) for v, sec in sections.items() if v <= top_section)],
                'encoded': text,
                'args': tuple(re.match(r'#w(\d+)h(\d+)p(\w+)', text).groups()),
            })
        else:
            sections[len(ident)] = text
            top_section = len(ident)
    return parsed

puzzles = parse_puzzles(puzzles_txt)

# build code
os.chdir(rs_dir)
subprocess.run(('cargo', 'build', '--release'))

# bench
durations = []
solved = 0
for puzzle in puzzles:
    print('trying to solve...', ' :: '.join(puzzle['section']))
    start = time.time()
    try:
        subprocess.run((str(executable), *puzzle['args']), check=True, timeout=BENCH_TIMEOUT, stdout=subprocess.DEVNULL)
        took = time.time() - start
        solved += 1
        print(f'success! found solution in {took:.3f}s')
    except subprocess.CalledProcessError:
        took = time.time() - start
        print(f'error! no solution in {took:.3f}s')
    except subprocess.TimeoutExpired:
        took = time.time() - start
        print(f'error! could not complete in {took:.3f}s')
    durations.append(took)

print(
    f'solved {solved}/{len(puzzles)} puzzles in {sum(durations):.3f}s'
    f' (mean={statistics.mean(durations):.3f}s,'
    f' median={statistics.median(durations):.3f}s,'
    f' max={max(durations):.3f}s,'
    f' min={min(durations):.3f}s)'
)
