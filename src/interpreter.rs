use std::fs::File;
use std::io::{self, Read, Write};
use std::collections::HashMap;

struct Lexer {
    position_in_code: usize,
    content: Vec<char>,
}

impl Lexer {
    pub fn new() -> Lexer {
        Lexer {
            position_in_code: 0,
            content: Vec::new(),
        }
    }

    pub fn fill(&mut self, code: &str) {
        for c in code.chars() {
            self.content.push(c);
        }
    }

    fn is_valid_brainfuck_instruction(&self, inst: char) -> bool {
        let valid = "><+-.,[]";
        if valid.contains(inst) {
            return true;
        } else {
            return false;
        }
    }

    pub fn next(&mut self) -> char {
        while self.position_in_code < self.content.len() && !self.is_valid_brainfuck_instruction(self.content[self.position_in_code]) {
            self.position_in_code += 1;
        }

        if self.position_in_code >= self.content.len() {
            return '@'; // EOF character, randomly chosen.
        }

        let r = self.content[self.position_in_code];
        self.position_in_code += 1;
        return r;
    }
}

#[derive(Clone, Copy, PartialEq)]
enum IRInstructionKind {
    IncrementPointer,
    DecrementPointer,
    IncrementByte,
    DecrementByte,
    PrintByteAsChar,
    ReadInputToByte,
    JumpIfZero,
    JumpIfNotZero,
}

#[derive(Clone, Copy)]
struct IRInstruction {
    kind: IRInstructionKind,
    operand: Option<u8>,
}

const RAM_SIZE: usize = 100_000;

pub struct Interpreter {
    memory_pointer: usize,
    instruction_pointer: usize,
    ram: [u8; RAM_SIZE],
    program: Vec<IRInstruction>,
    jump_map: HashMap<usize, usize>,
    lexer: Lexer,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            memory_pointer: 0,
            instruction_pointer: 0,
            ram: [0x0; RAM_SIZE],
            program: Vec::new(),
            jump_map: HashMap::new(),
            lexer: Lexer::new(),
        }
    }

    pub fn load_program_from_file(&mut self, program_path: &str) {
        let mut program_file = File::open(program_path).expect("[ERROR] Unable to open the file !");

        let mut program_buffer = String::new();

        program_file.read_to_string(&mut program_buffer).expect("[ERROR] Unable to read the file !");

        self.lexer.fill(program_buffer.as_str());

        let mut c = self.lexer.next();

        while c != '@' {
            let ir_inst: IRInstruction;
            let inst_kind: IRInstructionKind;

            match c {
                '>' | '<' | '+' | '-' => {
                    if c == '>' { inst_kind = IRInstructionKind::IncrementPointer; }
                    else if c == '<' { inst_kind = IRInstructionKind::DecrementPointer; }
                    else if c == '+' { inst_kind = IRInstructionKind::IncrementByte; }
                    else { inst_kind = IRInstructionKind::DecrementByte; }

                    let mut streak = 1u8;
                    let mut s = self.lexer.next();

                    while c == s {
                        streak += 1;
                        s = self.lexer.next();
                    }

                    ir_inst = IRInstruction { kind: inst_kind, operand: Some(streak) };

                    c = s;
                },
                '.' | ',' | '[' | ']' => {
                    if c == '.' { inst_kind = IRInstructionKind::PrintByteAsChar; }
                    else if c == ',' { inst_kind = IRInstructionKind::ReadInputToByte; }
                    else if c == '[' { inst_kind = IRInstructionKind::JumpIfZero; }
                    else { inst_kind = IRInstructionKind::JumpIfNotZero; }

                    ir_inst = IRInstruction { kind: inst_kind, operand: None };

                    c = self.lexer.next();
                },
                _ => continue,
            }
            self.program.push(ir_inst);
        }
    }

    fn precompute_jumps(&mut self) {
        let mut stack = Vec::<usize>::new();

        let mut local_instruction_pointer = 0usize;

        while local_instruction_pointer < self.program.len() {
            let inst = self.program[local_instruction_pointer];

            match inst.kind {
                IRInstructionKind::JumpIfZero => stack.push(local_instruction_pointer),
                IRInstructionKind::JumpIfNotZero => {
                    let target = stack.pop().unwrap();
                    self.jump_map.insert(local_instruction_pointer, target);
                    self.jump_map.insert(target, local_instruction_pointer);
                },
                _ => (), // Other instructions aren't jump related.
            }

            local_instruction_pointer += 1;
        }
    }

    pub fn interpret(&mut self) {
        self.precompute_jumps();

        while self.instruction_pointer < self.program.len() {
            let inst = self.program[self.instruction_pointer];

            match inst.kind {
                IRInstructionKind::IncrementPointer => self.memory_pointer += inst.operand.unwrap() as usize,
                IRInstructionKind::DecrementPointer => self.memory_pointer -= inst.operand.unwrap() as usize,
                IRInstructionKind::IncrementByte => self.ram[self.memory_pointer] += inst.operand.unwrap(),
                IRInstructionKind::DecrementByte => self.ram[self.memory_pointer] -= inst.operand.unwrap(),
                IRInstructionKind::PrintByteAsChar => {
                    let byte_as_char = self.ram[self.memory_pointer] as char;
                    print!("{byte_as_char}");
                    io::stdout().flush().unwrap();
                },
                IRInstructionKind::ReadInputToByte => {
                    let mut input: [u8; 1] = [0; 1];
                    io::stdin().read_exact(&mut input).expect("[ERROR] Unable to read stdin.");
                    self.ram[self.memory_pointer] = input[0];
                },
                IRInstructionKind::JumpIfZero => {
                    if self.ram[self.memory_pointer] == 0 {
                        self.instruction_pointer = *self.jump_map.get(&self.instruction_pointer).unwrap();
                    }
                },
                IRInstructionKind::JumpIfNotZero => {
                    if self.ram[self.memory_pointer] != 0 {
                        self.instruction_pointer = *self.jump_map.get(&self.instruction_pointer).unwrap();
                    }
                }
            }

            self.instruction_pointer += 1;
        }
    }
}
