import subprocess
import os
import re
import time
from pathlib import Path

BENCH_TIMEOUT = 30

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
for puzzle in puzzles:
    print('trying to solve...', ' :: '.join(puzzle['section']))
    start = time.time()
    try:
        subprocess.run((str(executable), *puzzle['args']), timeout=BENCH_TIMEOUT, stdout=subprocess.DEVNULL)
        print(f'success! took {time.time() - start:.3f}s')
    except subprocess.TimeoutExpired:
        print(f'could not complete after {time.time() - start:.3f}s!')
