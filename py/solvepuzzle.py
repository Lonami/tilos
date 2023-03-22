import sys

class Shape:
    def __init__(self, rows):
        self.rows = rows

    @classmethod
    def from_str(cls, string):
        rows = [[c != '_' for c in line if not c.isspace()] for line in string.strip().splitlines()]
        if not all(len(row) == len(rows[0]) for row in rows):
            raise ValueError('bad string is missing empty space')
        return cls(rows)

    @classmethod
    def with_size(cls, w, h):
        return cls([[False] * w for _ in range(h)])

    @property
    def w(self):
        return len(self.rows[0])

    @property
    def h(self):
        return len(self.rows)

    @property
    def full(self):
        return all(map(all, self.rows))

    def rot(self):
        return Shape(list(map(list, zip(*self.rows[::-1]))))

    def flip(self):
        return Shape([row[::-1] for row in self.rows])

    def put(self, x, y, other):
        if x < 0 or x + other.w > self.w or y < 0 or y + other.h > self.h:
            return None

        new = [row[:] for row in self.rows]
        for i in range(other.h):
            for j in range(other.w):
                if other.rows[i][j]:
                    if new[y + i][x + j]:
                        return None
                    new[y + i][x + j] = True

        return Shape(new)

    def __len__(self):
        return sum(sum(row) for row in self.rows)

    def __repr__(self):
        return f'Shape.from_str({str(self)!r})'

    def __str__(self):
        return '\n'.join(' '.join('_#'[c] for c in row) for row in self.rows)


shapes = {k: Shape.from_str(v) for k, v in {
    't': '''
         # # #
         _ # _
         ''',
    'i': '''
         # # # #
         ''',
    'l': '''
         # # #
         # _ _
         ''',
    'o': '''
         # #
         # #
         ''',
    's': '''
         _ # #
         # # _
         ''',
}.items()}

for key, shape in list(shapes.items()):
    shapes[key.upper()] = shape.flip().rot().rot()


def solve(board, pieces, used):
    if board.full:
        for x, y, piece, r in used:
            print(f'rotate {r} times, then place at ({x}, {y}):')
            print(f'{piece}')
        return True

    for y in range(board.h):
        for x in range(board.w):
            for i, piece in enumerate(pieces):
                for r in range(3):
                    if new := board.put(x, y, piece):
                        if solve(new, pieces[:i] + pieces[i + 1:], used + [(x, y, piece, r)]):
                            return True
                    piece = piece.rot()


def main(args):
    if len(args) != 3:
        print(f'usage: {sys.argv[0]} WIDTH HEIGHT PIECES', file=sys.stderr)
        print(f'available pieces (uppercase to flip it):', file=sys.stderr)
        for k, shape in shapes.items():
            if not k.islower():
                continue
            print(f'-------- {k}:', file=sys.stderr)
            print(f'{shape}', file=sys.stderr)
            print(file=sys.stderr)
        return 1

    try:
        w, h = map(int, args[:2])
        pieces = [shapes[k] for k in args[2]]
    except KeyError:
        print(f'bad piece given, allowed choices are: {"".join(shapes)}', file=sys.stderr)
        return 2
    except ValueError:
        print(f'bad dimensions given (not a number)', file=sys.stderr)
        return 3

    if sum(map(len, pieces)) != w * h:
        print(f'bad puzzle dimensions (given pieces are not enough or are too much)', file=sys.stderr)
        return 4

    solve(Shape.with_size(w, h), pieces, [])


if __name__ == '__main__':
    exit(main(sys.argv[1:]) or 0)
