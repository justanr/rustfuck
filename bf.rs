use std::char;
use std::num::Wrapping;

const TAPE_SIZE: usize = 30_000;

#[derive(Debug, Eq, PartialEq, Clone)]
enum BrainFuckToken {
    MoveRight,
    MoveLeft,
    Incr,
    Decr,
    Output,
    Input,
    JumpForward,
    JumpBackward
}

impl BrainFuckToken {
    fn from_char(c: char) -> Option<Self> {
         match c {
            '>' => Some(BrainFuckToken::MoveRight),
            '<' => Some(BrainFuckToken::MoveLeft),
            '+' => Some(BrainFuckToken::Incr),
            '-' => Some(BrainFuckToken::Decr),
            '.' => Some(BrainFuckToken::Output),
            ',' => Some(BrainFuckToken::Input),
            '[' => Some(BrainFuckToken::JumpForward),
            ']' => Some(BrainFuckToken::JumpBackward),
            _ => None
        }
    }
}


#[derive(Debug)]
struct Collapsed(BrainFuckToken, usize);

fn lex(prog: String) -> Vec<BrainFuckToken> {
    prog.chars().filter_map(BrainFuckToken::from_char).collect()
}


fn collapse(ops: Vec<BrainFuckToken>) -> Vec<Collapsed> {
    let mut loc: usize = 0;
    let mut collapsed = Vec::with_capacity(ops.len());
    let mut brackets = Vec::new();

    while let Some(symbol) = ops.get(loc) {
        match symbol {
            &BrainFuckToken::JumpForward => {
                brackets.push(collapsed.len());
                collapsed.push(Collapsed(BrainFuckToken::JumpForward, 0));
                loc += 1;
            },

            &BrainFuckToken::JumpBackward => {
                let idx = brackets.pop().unwrap_or_else(|| { panic!("Mismatched brackets: {}", loc); });

                collapsed[idx] = match collapsed.get(idx).unwrap_or_else(|| {
                    panic!("No tokens found at specified stack location: {}", idx);
                })
                {
                    &Collapsed(BrainFuckToken::JumpForward, 0) => {
                        Collapsed(BrainFuckToken::JumpForward, collapsed.len())
                    },
                    &Collapsed(BrainFuckToken::JumpForward, _) => {
                        panic!("Matched populated JumpForward");
                    },
                    tok => {
                        panic!("Matched token was not a JumpForward, got: {:?}", tok);
                    }
                };

                collapsed.push(Collapsed(BrainFuckToken::JumpBackward, idx));

                loc += 1;
            },

            sym @ _ => {
                let mut count = 0;

                loop {
                    match ops.get(loc) {
                        Some(sym) if sym == symbol => {
                            loc += 1;
                            count += 1;
                        },
                        _ => { break; }
                    }
                }

                collapsed.push(Collapsed(sym.clone(), count));
            }
        }
    }

    collapsed.shrink_to_fit();
    collapsed
}


fn run(instructions: Vec<Collapsed>, input: &String) -> String {
    let mut memory = [0u8; TAPE_SIZE];
    let mut instptr: usize = 0;
    let mut memptr: usize = 0;
    let mut input_iter = input.chars();
    let mut output = String::new();

    while let Some(instruction) = instructions.get(instptr) {

        match *instruction {
            Collapsed(BrainFuckToken::MoveRight, x) => {
                memptr = (memptr + x) % TAPE_SIZE;
            },

            Collapsed(BrainFuckToken::MoveLeft, x) => {
                memptr = (((memptr as i32) - (x as i32)) % TAPE_SIZE as i32) as usize;
            },

            Collapsed(BrainFuckToken::Incr, x) => {
                memory[memptr] = (Wrapping::<u8>(memory[memptr]) + Wrapping::<u8>(x as u8)).0;
            },

            Collapsed(BrainFuckToken::Decr, x) => {
                memory[memptr] = (Wrapping::<u8>(memory[memptr]) - Wrapping::<u8>(x as u8)).0;
            },

            Collapsed(BrainFuckToken::Output, x) => {
                let ord = memory[memptr] as u32;
                let chr = char::from_u32(ord).unwrap_or_else(|| panic!("Invalid character: {}", ord));
                for _ in 0..x {
                    output.push(chr);
                }
            },

            Collapsed(BrainFuckToken::Input, x) => {
                for _ in 0..x {
                    memory[memptr] = Wrapping::<u8>(input_iter.next().unwrap_or('\0') as u8).0;
                }
            },

            Collapsed(BrainFuckToken::JumpBackward, ptr) => {
                if memory[memptr] != 0 {
                    instptr = ptr;
                }
            },

            Collapsed(BrainFuckToken::JumpForward, ptr) => {
                if memory[memptr] == 0 {
                    instptr = ptr;
                }
            },
        }
        instptr += 1;
    }

    output
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
    let input = String::new();

    let lexed = lex(s);
    let tokens = collapse(lexed);
    let result = run(tokens, &input);
    println!("{}", result);
}
