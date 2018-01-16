from enum import Enum
from typing import NamedTuple
from io import StringIO
from pprint import pprint


class BFToken(Enum):
    MoveRight = '>'
    MoveLeft = '<'
    JumpForward = '['
    JumpBackward = ']'
    Incr = '+'
    Decr = '-'
    ReadStdIn = ','
    PutStdOut = '.'

    @classmethod
    def get(cls, token):
        try:
            return cls(token)
        except ValueError:
            return None

    def __repr__(self):
        return f"{self.__class__.__name__}.{self.name}"


class CollapsedToken(NamedTuple):
    token: BFToken
    value: int


def to_u8(value):
    return value % 255


class MismatchedBrackets(SyntaxError):
    pass

def parse(prog):
    loc = 0
    length = len(prog)
    tokens = []
    bracket_stack = []  # idx

    while loc < length:
        symbol = prog[loc]
        op = BFToken.get(symbol)
        loc += 1

        if op is None:
            continue

        if op == BFToken.JumpBackward:
            if not bracket_stack:
                raise MismatchedBrackets(loc)

            open_idx = bracket_stack.pop()
            opened = tokens[open_idx]

            if opened.token != BFToken.JumpForward or opened.value is not None:
                raise MismatchedBrackets(f"Expected to find empty JumpForward found {opened}")

            tokens[open_idx] = opened._replace(value=len(tokens))

            tokens.append(CollapsedToken(token=op, value=open_idx))

        if op == BFToken.JumpForward:
            bracket_stack.append(len(tokens))
            tokens.append(CollapsedToken(token=op, value=None))

        if op not in {BFToken.JumpBackward, BFToken.JumpForward}:
            count = 1
            while loc < length and symbol == prog[loc]:
                count += 1
                loc += 1

            tokens.append(CollapsedToken(token=op, value=count))

    if bracket_stack:
        raise MismatchedBrackets(bracket_stack.pop())

    return tokens


TAPE_SIZE = 30000


def run(ops, stdin, stdout):
    tape = [0] * TAPE_SIZE
    loc = 0
    ptr = 0
    stdin_iter = iter(stdin)
    prog_length = len(ops)

    while loc < prog_length:
        op = ops[loc]

        if op.token == BFToken.MoveRight:
            move_to = op.value + ptr
            if move_to > TAPE_SIZE - 1:
                move_to %= TAPE_SIZE
            ptr = move_to

        if op.token == BFToken.MoveLeft:
            move_to = ptr - op.value
            if move_to < 0:
                move_to %= TAPE_SIZE
            ptr = move_to

        if op.token == BFToken.Incr:
            tape[ptr] = to_u8(tape[ptr] + op.value)

        if op.token == BFToken.Decr:
            tape[ptr] = to_u8(tape[ptr] - op.value)

        if op.token == BFToken.ReadStdIn:
            for _ in range(op.value):
                tape[ptr] = ord(next(stdin_iter, '\0'))

        if op.token == BFToken.PutStdOut:
            for _ in range(op.value):
                stdout.write(chr(tape[ptr]))

        if op.token == BFToken.JumpForward:
            if tape[ptr] == 0:
                loc = op.value

        if op.token == BFToken.JumpBackward:
            if tape[ptr] != 0:
                loc = op.value

        loc += 1


def bf_to_one_line(prog):
    return "".join(prog.split('\n'))


def run_bf(prog, stdout, stdin=""):
    run(parse(bf_to_one_line(prog)), stdin, stdout)


if __name__ == "__main__":
    import sys

    def main():

        if len(sys.argv) == 1:
            sys.stderr.write("Expected at least one argument, a BrainFuck program and optionally input")
            sys.exit(1)

        try:
            with open(sys.argv[1]) as fh:
                prog = fh.read()
        except OSError:
            sys.stderr.write(f"Cannot read {sys.argv[1]}")

        stdin = sys.argv[2] if len(sys.argv) > 2 else ""
        stdout = StringIO()

        try:
            run_bf(prog, stdout, stdin)
        except:
            stdout.seek(0)
            sys.stdout.write("\n")
            sys.stdout.write("Content so far: ")
            sys.stdout.write(stdout.read())
            sys.stdout.write("\n")
            raise

        stdout.seek(0)
        sys.stdout.write('\n'+ stdout.read() + '\n')

    main()
