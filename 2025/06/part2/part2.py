from collections.abc import Iterable
import functools
import itertools
import operator
from pathlib import Path
import sys
from pprint import pprint

_OPS_MAP = {'*': operator.mul, '+': operator.add}

def main(input_file: Path):
    with input_file.open('r') as f:
        lines = f.readlines()
        cols = zip(*lines)
        entries = (
            (from_numdigits(col[:-1]), _OPS_MAP.get(col[-1], None))
            for col in cols
        )
        problems = (
            (
                # the operator
                entry[1],
                itertools.chain(
                    # first number, which came alongside the operator
                    (entry[0],),
                    # subsequent numbers (take all non-None numbers from entries, and consume but don't return the terminating None)
                    itertools.takewhile(lambda n: n is not None, (n for n,_ in entries))
                )
            )
            for entry in entries
        )
        solutions = (
            functools.reduce(op, nums)
            for op, nums in problems
        )
        answer = sum(solutions)
        print(f'{answer=}')

def from_numdigits(digits: Iterable[str]) -> int | None:
    cur = None
    for digit in digits:
        match digit:
            case ' ' | '\n': pass
            case n:
                if cur is None: cur = 0
                cur *= 10
                cur += int(n)
    return cur

if __name__ == '__main__':
    files = sys.argv[1:]
    if len(files) == 0:
        files = ['sample.txt', 'input.txt']
    for p in map(Path, files):
        print(f'{p}')
        main(p)
