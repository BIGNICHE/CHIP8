use std::{fs, io, env, path};
static MEM_SIZE:usize = 0x1000; // Size of CHIP-8 Memory.
static PROG_START:i32 = 0x200;
static MAX_PROGRAM_SIZE:usize = 0xE00;
const DISPLAY_WIDTH:usize = 64;
const DISPLAY_HEIGHT:usize = 32;

/*
struct Stack<T> {
    stack: Vec<T>,
}

impl<T> Stack<T> {
    fn new() -> Self {
      Stack { stack: Vec::new() }
    }
    fn pop(&mut self) -> Option<T> {
        self.stack.pop()
    }
      
    fn push(&mut self, item: T) {
        self.stack.push(item)
    }
  }

   */
struct Display {
    pixels: [[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT ],
}

impl Display {
    fn draw(&self) {
        // draws to console


    } 



}


fn emplace_at(data: u8, emplace_idx: usize , memory: &mut Vec<u8>)
{

    memory[emplace_idx] = data;

}

fn vec_emplace_at(data: Vec<u8>, emplace_idx: usize, memory: &mut Vec<u8>)
{

    let mut mem_ptr = emplace_idx.clone();

    let mut data_it = data.into_iter().peekable();
    while data_it.peek().is_some() {
        
        memory[mem_ptr] = data_it.next().unwrap();
        mem_ptr += 1;

    }

}




fn load_program(path: path::PathBuf, memory: &mut Vec<u8>) {

    let contents = fs::read(path)
        .expect("Could not read file.");

    if contents.len() > MAX_PROGRAM_SIZE {
        // return error.
    }

    let mut file_it = contents.into_iter().peekable();

    let mut write_ptr = PROG_START;

    while file_it.peek().is_some() {
        
        memory[write_ptr as usize] = file_it.next().unwrap();
        write_ptr += 1;

    }

    // Basic font for writing characters
    let font: Vec<u8> = vec![
        0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
        0x20, 0x60, 0x20, 0x20, 0x70, // 1
        0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
        0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
        0x90, 0x90, 0xF0, 0x10, 0x10, // 4
        0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
        0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
        0xF0, 0x10, 0x20, 0x40, 0x40, // 7
        0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
        0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
        0xF0, 0x90, 0xF0, 0x90, 0x90, // A
        0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
        0xF0, 0x80, 0x80, 0x80, 0xF0, // C
        0xE0, 0x90, 0x90, 0x90, 0xE0, // D
        0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
        0xF0, 0x80, 0xF0, 0x80, 0x80  // F
    ];

    vec_emplace_at(font, 0x050, memory)

}

fn hex_dump(memory: Vec<u8>) {

    println!("{:x?}", memory);

}



fn main() {

    let mut memory: Vec<u8> = vec![0; MEM_SIZE];
    let mut display: Display = Display{pixels: [[false; DISPLAY_WIDTH]; DISPLAY_HEIGHT]};

    //let mut rom_path = exec_path.clone();
    let mut rom_path = path::PathBuf::from(r"C:\Users\Nick\Documents\GitHub\RustCHIP8");
    rom_path.push("roms");
    rom_path.push(r"IBM Logo.ch8");

    let mut stack: Vec<u16>;
    let mut pc: u16 = PROG_START as u16; // Program counter
    

    // Init registers.
    // These are a simulacrum of a cpu's internal registers.

    let (mut I, mut V0, mut V1,
         mut V2, mut V3, mut V4,
         mut V5, mut V6, mut V7,
         mut V8, mut V9, mut VA,
         mut VB, mut VC, mut VD,
         mut VE, mut VF) = 
         (0u16, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8);



    load_program(rom_path, &mut memory);

    







    



    hex_dump(memory);

}
