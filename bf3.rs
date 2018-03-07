use std::collections::{HashMap, VecDeque};
use std::fmt::{Display, Formatter, Result, Write};
use std::mem::replace;
use std::iter::FromIterator;

const TAPE_SIZE: i32 = 30000;
type JumpLocs = (usize, usize);
type Tokens = Vec<BrainFuckToken>;

#[derive(Debug, Clone, Copy)]
enum BrainFuckToken {
    Move(isize),
    JumpF(usize),
    JumpB(usize),
    Incr(i32),
    StdOut,
    StdIn,
    ZeroOut,
}


impl Display for BrainFuckToken {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            &BrainFuckToken::Move(x) => write!(f, " M{}", &x),
            &BrainFuckToken::JumpF(_) => write!(f, " ["),
            &BrainFuckToken::JumpB(_) => write!(f, " ]"),
            &BrainFuckToken::Incr(x) => write!(f, " I{}", &x),
            &BrainFuckToken::StdOut => write!(f, "O"),
            &BrainFuckToken::StdIn => write!(f, " I"),
            &BrainFuckToken::ZeroOut => write!(f, " @"),
        }
    }
}

#[derive(Debug)]
struct Trace {
    count: HashMap<JumpLocs, u32>,
}

impl Trace {
    fn new() -> Trace {
        Trace {
            count: HashMap::new(),
        }
    }

    fn reset(&mut self) {
        self.count = HashMap::new();
    }

    fn trace(&mut self, locs: JumpLocs) {
        let c = self.count.entry(locs).or_insert(0);
        *c += 1;
    }

    fn report(&mut self, prog: &Vec<BrainFuckToken>) -> HashMap<String, u32> {
        let mut report: HashMap<String, u32> = HashMap::new();
        for (name, c) in self.count
            .iter()
            .filter(|&(_, c)| *c > 100)
            .map(|(locs, c)| (token_run_to_string(locs, prog), c))
        {
            let e = report.entry(name).or_insert(0);
            *e += c;
        }

        report
    }
}

fn token_run_to_string(locs: &JumpLocs, ops: &Tokens) -> String {
    let (start, finish) = *locs;
    let mut s = String::with_capacity(finish - start + 1);

    for token in &ops[start..finish + 1] {
        write!(s, "{}", token).ok();
    }

    s
}

struct Tape {
    loc: usize,
    tape: [i32; 30000],
}

impl Tape {
    fn new() -> Tape {
        Tape {
            loc: 0,
            tape: [0i32; 30000],
        }
    }

    fn move_(&mut self, move_: isize) {
        let spaces = self.loc as i32 + move_ as i32;
        self.loc = (spaces % TAPE_SIZE) as usize;
    }

    fn incr(&mut self, inc: i32) {
        self.tape[self.loc] += inc;
    }

    fn get(&self) -> i32 {
        self.tape[self.loc]
    }

    fn getc(&self) -> char {
        self.get() as u8 as char
    }

    fn put(&mut self, x: i32) {
        self.tape[self.loc] = x;
    }

    fn putc(&mut self, c: char) {
        self.tape[self.loc] = c as i32;
    }
}

struct Program {
    loc: usize,
    ops: Vec<BrainFuckToken>,
    tape: Tape,
    tracer: Trace,
}

impl Program {
    fn new(ops: Vec<BrainFuckToken>) -> Program {
        Program {
            loc: 0,
            ops: ops,
            tape: Tape::new(),
            tracer: Trace::new(),
        }
    }

    fn run(&mut self, input: String, out: &mut String) {
        self.tracer.reset();
        let mut input_iter = input.chars();

        while let Some(instr) = self.ops.get(self.loc) {
            match *instr {
                BrainFuckToken::JumpF(x) => {
                    if self.tape.get() == 0 {
                        self.loc = x;
                    } else {
                        self.tracer.trace((self.loc, x));
                    }
                }
                BrainFuckToken::JumpB(x) => {
                    if self.tape.get() != 0 {
                        self.loc = x;
                    }
                }
                BrainFuckToken::Move(x) => self.tape.move_(x),
                BrainFuckToken::Incr(x) => self.tape.incr(x),
                BrainFuckToken::StdIn => self.tape.putc(input_iter.next().unwrap_or('\0')),
                BrainFuckToken::StdOut => out.push(self.tape.getc()),
                BrainFuckToken::ZeroOut => self.tape.put(0),
            }
            self.loc += 1;
        }
    }
}

impl BrainFuckToken {
    pub fn from_char(c: char) -> Option<BrainFuckToken> {
        match c {
            '+' => Some(BrainFuckToken::Incr(1)),
            '-' => Some(BrainFuckToken::Incr(-1)),
            '>' => Some(BrainFuckToken::Move(1)),
            '<' => Some(BrainFuckToken::Move(-1)),
            '.' => Some(BrainFuckToken::StdOut),
            ',' => Some(BrainFuckToken::StdIn),
            '[' => Some(BrainFuckToken::JumpF(0)),
            ']' => Some(BrainFuckToken::JumpB(0)),
            _ => None,
        }
    }
}

fn parse<T>(source: T) -> VecDeque<BrainFuckToken>
where
    T: Iterator<Item = char>,
{
    VecDeque::from_iter(source.filter_map(BrainFuckToken::from_char))
}

fn optimize(tokens: VecDeque<BrainFuckToken>) -> Vec<BrainFuckToken> {
    let mut program = handle_zero_out(collapse_tokens(tokens));
    build_jumps(&mut program);
    program.into()
}

fn collapse_tokens(mut tokens: VecDeque<BrainFuckToken>) -> VecDeque<BrainFuckToken> {
    let mut program = VecDeque::new();

    while let Some(token) = tokens.pop_front() {
        if program.len() == 0 {
            program.push_back(token);
            continue;
        }

        let previous = program.pop_back().unwrap();

        match (previous, token) {
            (BrainFuckToken::Incr(x), BrainFuckToken::Incr(y)) => {
                let v = x + y;
                if v != 0 {
                    program.push_back(BrainFuckToken::Incr(v));
                }
            }
            (BrainFuckToken::Move(x), BrainFuckToken::Move(y)) => {
                let v = x + y;
                if v != 0 {
                    program.push_back(BrainFuckToken::Move(v));
                }
            }
            _ => {
                program.push_back(previous);
                program.push_back(token);
            }
        }
    }

    program
}

fn handle_zero_out(mut tokens: VecDeque<BrainFuckToken>) -> Vec<BrainFuckToken> {
    let mut program = Vec::new();

    while let Some(token) = tokens.pop_front() {
        program.push(token);

        if program.len() < 3 {
            continue;
        }

        let (third, second, first) = (
            program.pop().unwrap(),
            program.pop().unwrap(),
            program.pop().unwrap(),
        );

        match (first, second, third) {
            (BrainFuckToken::JumpF(_), BrainFuckToken::Incr(x), BrainFuckToken::JumpB(_))
                if x < 0 =>
            {
                program.push(BrainFuckToken::ZeroOut);
            }
            _ => {
                program.push(first);
                program.push(second);
                program.push(third);
            }
        }
    }

    program
}

fn build_jumps(tokens: &mut Vec<BrainFuckToken>) {
    let mut brackets = Vec::new();

    for idx in 0..tokens.len() {
        match tokens[idx] {
            BrainFuckToken::JumpF(_) => brackets.push(idx),
            BrainFuckToken::JumpB(_) => {
                let partner = brackets
                    .pop()
                    .unwrap_or_else(|| panic!("unmatched bracket at {}", idx));
                replace(&mut tokens[idx], BrainFuckToken::JumpB(partner));
                replace(&mut tokens[partner], BrainFuckToken::JumpF(idx));
            }
            _ => {}
        }
    }

    if brackets.len() != 0 {
        panic!("Unmatched brackets at: {:?}", brackets);
    }
}

fn main() {
    use std::fs::File;
    use std::path::Path;
    use std::io::prelude::*;
    use std::env;

    let arg1 = env::args().nth(1).unwrap();
    let path = Path::new(&arg1);
    let mut s = String::new();
    let mut file = File::open(&path).unwrap();
    file.read_to_string(&mut s).unwrap();

    let tokens = optimize(parse(s.chars()));
    let mut prog = Program::new(tokens);
    let input = String::new();
    let mut output = String::new();
    prog.run(input, &mut output);
    println!("Output:\n{}", output);

    println!("\nTrace:\n");
    let r = prog.tracer.report(&prog.ops);

    let mut report: Vec<(&String, &u32)> = r.iter().collect();
    report.sort_by(|&(_, a), &(_, b)| b.cmp(a));

    for (name, count) in report {
        println!("{} -> {}", name, count);
    }
}
