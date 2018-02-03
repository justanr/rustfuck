use std::collections::{HashMap, VecDeque};
use std::fmt::{Display, Formatter, Result, Write};

const TAPE_SIZE: usize = 30000;
type AST = VecDeque<BrainFuckAST>;
type BrainFuckProgram = Vec<BrainFuckOpCode>;
type Jump = (usize, usize);
type JumpTable = HashMap<usize, usize>;
type TraceReport = HashMap<String, u32>;


#[derive(Debug, PartialEq, Eq)]
enum BrainFuckAST {
    MoveR,
    MoveL,
    Incr,
    Decr,
    Loop(AST),
    StdIn,
    StdOut
}


#[derive(Debug)]
enum BrainFuckOpCode {
    Move(isize),
    Incr(i32),
    JumpF,
    JumpB,
    StdOut,
    StdIn,
    ZeroOut,
}


impl Display for BrainFuckOpCode {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            BrainFuckOpCode::Move(x) => { write!(f, " M{}", &x) },
            BrainFuckOpCode::Incr(x) => { write!(f, " I{}", &x) },
            BrainFuckOpCode::JumpF   => { write!(f, " [") },
            BrainFuckOpCode::JumpB   => { write!(f, " ]") },
            BrainFuckOpCode::StdOut  => { write!(f, " O") },
            BrainFuckOpCode::StdIn   => { write!(f, " I") },
            BrainFuckOpCode::ZeroOut => { write!(f, " @") },
        }
    }
}

#[derive(Debug)]
struct Trace(HashMap<Jump, u32>);

struct Tape {
    loc: usize,
    tape: [i32; TAPE_SIZE]
}

struct Program {
    loc: usize,
    ops: BrainFuckProgram,
    jumps: JumpTable,
    tape: Tape,
    trace: Trace
}


impl Trace {
    fn new() -> Trace {
        Trace(HashMap::new())
    }

    fn reset(&mut self) {
        self.0 = HashMap::new();
    }

    fn trace(&mut self, locs: Jump) {
        let c = self.0.entry(locs).or_insert(0);
        *c += 1;
    }

    fn report(&mut self, prog: &BrainFuckProgram) -> TraceReport {
        let mut report = TraceReport::new();
        for (name, c) in self.0
                .iter()
                .filter(|&(_, x)| *x > 100)
                .map(|(locs, c)| { (token_run_to_string(&locs, &prog), c)})
        {
            let e = report.entry(name).or_insert(0);
            *e += c;
        }

        report
    }
}


impl Tape {
    fn new() -> Tape {
        Tape {
            loc: 0,
            tape: [0i32; TAPE_SIZE]
        }
    }

    fn move_(&mut self, how_far: isize) {
        let spaces = self.loc as i32 + how_far as i32;
        self.loc = (spaces % TAPE_SIZE as i32) as usize;
    }

    fn incr(&mut self, inc: i32) {
        self.tape[self.loc] += inc;
    }

    fn zero(&mut self) {
        self.tape[self.loc] = 0;
    }

    fn get(&self) -> i32 {
        self.tape[self.loc]
    }

    fn getc(&self) -> char {
        self.get() as u8 as char
    }

    fn put(&mut self, c: char) {
        self.tape[self.loc] = c as i32;
    }
}

impl Program {
    fn new(ops: BrainFuckProgram, jumps: JumpTable) -> Program {
        Program {
            loc: 0,
            ops: ops,
            jumps: jumps,
            tape: Tape::new(),
            trace: Trace::new()
        }
    }

    fn run(&mut self, input: String, out: &mut String) {
        self.trace.reset();
        let mut input_iter = input.chars();

        while let Some(instr) = self.ops.get(self.loc) {
            match *instr {
                BrainFuckOpCode::Move(x) => { self.tape.move_(x); },
                BrainFuckOpCode::Incr(x) => { self.tape.incr(x); },
                BrainFuckOpCode::StdIn => { self.tape.put(input_iter.next().unwrap_or('\0')); },
                BrainFuckOpCode::StdOut => { out.push(self.tape.getc()); },
                BrainFuckOpCode::ZeroOut => { self.tape.zero(); },
                BrainFuckOpCode::JumpF => {
                    let partner = self.jumps.get(&self.loc)
                        .unwrap_or_else(|| {
                            println!("Current Loc: {}", self.loc);
                            println!("{:#?}", &self.jumps);
                            println!("{:#?}", &self.ops);
                            panic!("couldnt find JumpF partner")
                        });
                    if self.tape.get() == 0 {
                        self.loc = *partner;
                    } else {
                        self.trace.trace((self.loc, *partner));
                    }
                },
                BrainFuckOpCode::JumpB => {
                    let partner = self.jumps.get(&self.loc)
                        .unwrap_or_else(|| {
                            println!("Current Loc: {}", self.loc);
                            println!("{:#?}", &self.jumps);
                            println!("{:#?}", &self.ops);
                            panic!("couldnt find JumpB partner");
                        }
                        );
                    if self.tape.get() != 0 {
                        self.loc = *partner;
                    }
                },
            }
            self.loc += 1;
        }
    }
}

fn parse<T>(raw: &mut T) -> AST
    where T: Iterator<Item=char>
{
    let mut tokens = AST::new();

    while let Some(op) = raw.next() {
        match op {
            '+' => { tokens.push_back(BrainFuckAST::Incr); },
            '-' => { tokens.push_back(BrainFuckAST::Decr); },
            '>' => { tokens.push_back(BrainFuckAST::MoveR); },
            '<' => { tokens.push_back(BrainFuckAST::MoveL); },
            '.' => { tokens.push_back(BrainFuckAST::StdOut); },
            ',' => { tokens.push_back(BrainFuckAST::StdIn); },
            '[' => { tokens.push_back(BrainFuckAST::Loop(parse(raw))); },
            ']' => { return tokens; },
            _ => {}
        }
    }

    tokens
}


fn optimize(mut ast: AST) -> BrainFuckProgram {
    let mut program = BrainFuckProgram::new();

    while let Some(node) = ast.pop_front() {
        match node {
            BrainFuckAST::Loop(nodes) => {
                program.append(&mut compress_loop(nodes));
            },
            BrainFuckAST::StdOut => {
                program.push(BrainFuckOpCode::StdOut);
            },
            BrainFuckAST::StdIn => {
                program.push(BrainFuckOpCode::StdIn);
            },
            _ => {
                collapse(node, &mut ast, &mut program);
            }
        }
    }

    program
}



fn collapse(search: BrainFuckAST, mut ast: &mut AST, program: &mut BrainFuckProgram) {
    match search {
        BrainFuckAST::Incr => {
        let result = calculate_collapse(search, BrainFuckAST::Decr, &mut ast);
        if result != 0 {
            program.push(BrainFuckOpCode::Incr(result));
        }
        },
        BrainFuckAST::Decr => {
            let result = calculate_collapse(search, BrainFuckAST::Incr, &mut ast);
            program.push(BrainFuckOpCode::Incr(-result));
        },
        BrainFuckAST::MoveR => {
            let result = calculate_collapse(search, BrainFuckAST::MoveL, &mut ast) as isize;
            program.push(BrainFuckOpCode::Move(result));
        },
        BrainFuckAST::MoveL => {
            let result = calculate_collapse(search, BrainFuckAST::MoveR, &mut ast) as isize;
            program.push(BrainFuckOpCode::Move(-result));
        },
        _ => {}
    }

}

fn calculate_collapse(search: BrainFuckAST, opposite: BrainFuckAST, ast: &mut AST) -> i32 {
    let mut count = 1;

    while let Some(node) = ast.pop_front() {
        match node {
            _ if node == search => {
                count += 1;
            },
            _ if node == opposite => {
                count -= 1;
            }
            _ => {
                ast.push_front(node);
                break;
            }
        }

    }

    count
}


fn compress_loop(ast: AST) -> BrainFuckProgram {
    if ast.is_empty() {
        return vec![];
    }

    let mut sub = BrainFuckProgram::with_capacity(ast.len());;

    if ast.len() == 1 {
        match ast.get(0) {
            Some(&BrainFuckAST::Decr) => {
                sub.push(BrainFuckOpCode::ZeroOut);
                return sub;
            },
            _ => {}
        }
    }

    sub.push(BrainFuckOpCode::JumpF);
    sub.append(&mut optimize(ast));
    sub.push(BrainFuckOpCode::JumpB);
    sub.shrink_to_fit();
    sub
}


fn token_run_to_string(locs: &Jump, ops: &BrainFuckProgram) -> String {
    let (start, finish) = *locs;
    let mut s = String::with_capacity(finish - start + 1);

    for token in &ops[start..finish+1] {
        write!(s, "{}", token).ok();
    }

    s
}


fn build_jump_table(ops: &BrainFuckProgram) -> JumpTable {
    let mut jumps = JumpTable::new();
    let mut brackets = Vec::new();

    for (i, op) in ops.iter().enumerate() {
        match *op {
            BrainFuckOpCode::JumpF => {
                brackets.push(i);
            },
            BrainFuckOpCode::JumpB => {
                let partner = brackets.pop().unwrap();
                jumps.insert(i, partner);
                jumps.insert(partner, i);
            },
            _ => {}
        }
    }

    jumps
}


fn make_program(raw: String) -> Program {
    let parsed = parse(&mut raw.chars());
    let ops = optimize(parsed);
    let jumps = build_jump_table(&ops);

    Program::new(ops, jumps)
}


fn main() {
    use std::fs::File;
    use std::path::Path;
    use std::io::prelude::*;
    use std::env;

    let arg1 = env::args().nth(1).unwrap_or_else(|| panic!("not enough args"));
    let path = Path::new(&arg1);
    let mut s = String::new();
    let mut file = File::open(&path).unwrap_or_else(|_| panic!("couldnt open filed"));
    file.read_to_string(&mut s).unwrap_or_else(|_| panic!("couldnt read file"));;

    let mut prog = make_program(s);
    let input = String::new();
    let mut output = String::new();
    prog.run(input, &mut output);
    println!("Output:\n{}", output);
    let r = prog.trace.report(&prog.ops);
    let mut report: Vec<(&String, &u32)> = r.iter().collect();
    report.sort_by(|&(_, a), &(_, b)| { b.cmp(a) });

    println!("\nTrace\n");
    for (name, count) in report {
        println!("{} -> {}", name, count);
    }
}
