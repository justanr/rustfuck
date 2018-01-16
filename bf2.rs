const TAPE_SIZE: i32 = 30000;

#[derive(Debug)]
enum BrainFuckToken {
    Move(isize),
    JumpF(usize),
    JumpB(usize),
    Incr(i32),
    StdOut,
    StdIn,
}

use BrainFuckToken::*;

struct Tape {
    loc: usize,
    tape: [i32; 30000]
}


impl Tape {
    fn new() -> Tape {
        Tape {
            loc: 0,
            tape: [0i32; 30000]
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

    fn put(&mut self, c: char) {
        self.tape[self.loc] = c as i32;
    }
}


struct Program {
    loc: usize,
    ops: Vec<BrainFuckToken>,
    tape: Tape,
}

impl Program {
    fn new(ops: Vec<BrainFuckToken>) -> Program {
        Program {
            loc: 0,
            ops: ops,
            tape: Tape::new(),
        }
    }

    fn run(&mut self, input: String, out: &mut String) {
        let mut input_iter = input.chars();

        while let Some(instr) = self.ops.get(self.loc) {
            match *instr {
                JumpF(x) => {
                    if self.tape.get() == 0 {
                        self.loc = x;
                    } else {
                    }
                },
                JumpB(x) => {
                    if self.tape.get() != 0 {
                        self.loc = x;
                    }
                },
                Move(x) => {
                    self.tape.move_(x)
                },
                Incr(x) => {
                    self.tape.incr(x)
                },
                StdIn => {
                    self.tape.put(input_iter.next().unwrap_or('\0'))
                },
                StdOut => {
                    out.push(self.tape.getc());
                }
            }
            self.loc += 1;
        }
    }

    fn parse(prog: Vec<char>) -> Program {
        let mut tokens = Vec::with_capacity(prog.len());
        let mut brackets = Vec::new();
        let mut loc: usize = 0;

        while let Some(&symbol) = prog.get(loc) {
            match symbol {
                '[' => {
                    brackets.push(tokens.len());
                    tokens.push(JumpF(0));
                    loc += 1;
                },

                ']' => {
                    let idx = brackets.pop().unwrap_or_else(|| { panic!("Mismatched jumps"); });
                    tokens[idx] = match tokens.get(idx).unwrap_or_else(|| {panic!("Mismatched jumps")}) {
                        &JumpF(0) => {
                            JumpF(tokens.len())
                        },
                        &JumpF(_) => {
                            panic!("Matched populated Jump");
                        },
                        sym => { panic!("Expected jump, found {:?}", &sym); }
                    };
                    tokens.push(JumpB(idx));
                    loc += 1;
                },

                s @ '+' | s @ '-' => {
                   let (count, spaces) = find_token_run(s, &prog, loc);
                   tokens.push(if s == '+' { Incr(count) } else { Incr(-count) });
                   loc = spaces;
                },

                s @ '>' | s @ '<' => {
                    let (count, spaces) = find_token_run(s, &prog, loc);
                    tokens.push(if s == '>' { Move(count as isize) } else { Move(-count as isize) });
                    loc = spaces;
                },
                '.' => {
                    tokens.push(StdOut);
                    loc += 1;
                },
                ',' => {
                    tokens.push(StdIn);
                    loc += 1;
                }

                _ => {
                    loc += 1;
                }
            };
        }

        if brackets.len() > 0 {
            panic!("Mismatched brackets, last found: {}", brackets.pop().unwrap());
        }

        tokens.shrink_to_fit();
        Program::new(tokens)
    }
}

fn find_token_run(symbol: char, ops: &Vec<char>, start: usize) -> (i32, usize) {
    let mut loc = start;
    let mut count = 0;

    loop {
        match ops.get(loc) {
            Some(&sym) if sym == symbol => {
                loc += 1;
                count += 1;
            },
            _ => {
                break;
            }
        }
    }

    (count, loc)
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

    let tokens: Vec<char> = s.chars().collect();
    let mut prog = Program::parse(tokens);
    let input = String::new();
    let mut output = String::new();
    prog.run(input, &mut output);
    println!("{}", output);
}
