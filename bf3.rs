use std::collections::{HashMap, VecDeque};
use std::iter::Iterator;
use std::convert::From;


type AST = VecDeque<BrainFuckAST>;
type BrainFuckProgram = Vec<BrainFuckOpCode>;
type Jump = (usize, usize);
type JumpTable = HashMap<usize, usize>;
type TraceTable = HashMap<Jump, u32>;


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
    MaxOut,
    ConsumeInput,
    MoveReg(isize),
    NextZero(isize),
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
    let mut jumps = JumpTable::new();

    while let Some(node) = ast.pop_front() {
        match node {
            BrainFuckAST::Loop(nodes) => {
                let start = program.len();
                program.append(&mut compress_loop(nodes));
                let end = program.len();
                jumps.insert(start, end);
                jumps.insert(end, start);

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
    let mut sub = BrainFuckProgram::with_capacity(ast.len());;

    if ast.is_empty() {
        return sub;
    }

    if ast.len() == 1 {
        match ast.get(0) {
            Some(&BrainFuckAST::Decr) => {
                sub.push(BrainFuckOpCode::ZeroOut);
                return sub;
            },
            Some(&BrainFuckAST::Incr) => {
                sub.push(BrainFuckOpCode::MaxOut);
                return sub;
            },
            Some(&BrainFuckAST::MoveR) => {
                sub.push(BrainFuckOpCode::NextZero(1));
                return sub;
            },
            Some(&BrainFuckAST::MoveL) => {
                sub.push(BrainFuckOpCode::NextZero(-1));
                return sub;
            },
            Some(&BrainFuckAST::StdIn) => {
                sub.push(BrainFuckOpCode::ConsumeInput);
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


fn is_move(ast: &AST) -> bool {
    false
}


fn main() {
    use std::fs::File;
    use std::path::Path;
    use std::io::prelude::*;
    use std::io::{stdout, stderr};
    use std::env;

    let mut out = stdout();
    let mut err = stderr();

    let arg1 = env::args().nth(1).unwrap();
    let path = Path::new(&arg1);
    let mut s = String::new();
    let mut file = File::open(&path).unwrap();
    file.read_to_string(&mut s).unwrap();

    let output = parse(&mut s.chars());
    write!(out, "{:#?}", &output).ok();
    write!(err, "{:#?}", optimize(output)).ok();
}
