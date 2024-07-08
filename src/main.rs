use std::{env, fs, io, panic::PanicInfo, path};
static MEM_SIZE:usize = 0x1000; // Size of CHIP-8 Memory.
static PROG_START:i32 = 0x200;
static MAX_PROGRAM_SIZE:usize = 0xE00;
static FONT_MEMORY_START: i32 = 0x50;
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

/**
 * CHIP-8 cpu instruction, always 16 bits.
 */
struct Instruction {
    instruction: u16
}

impl Instruction {
    fn new(inst: u16) -> Self {
        Instruction { instruction: inst }
    }
    fn get_nibble(&mut self, nibble_index: usize) -> u8 {

        // Panics if value is not a u8. Since it is being logcal anded with 1111, it should fit within 4 bits.
        let nibble: u8 = ((self.instruction >> 4*(3 - nibble_index)) & 0xF).try_into().unwrap();

        return nibble;
    }

}


struct Display {
    pixels: [[u8; DISPLAY_WIDTH]; DISPLAY_HEIGHT ],
}

impl Display {
    fn print(&mut self) {
        // draws to console
        for j in 0..DISPLAY_WIDTH {
            for i in 0..DISPLAY_HEIGHT {
                //self.pixels[i][j] = false;
                if (self.pixels[i][j] > 0) {
                    print!("\u{2518}");
                } else {
                    print!(" ");
                }
            }
            print!("\n")
        }
        

    } 
    fn clear(&mut self) {

        for i in 0..DISPLAY_WIDTH {
            for j in 0..DISPLAY_HEIGHT {
                self.pixels[i][j] = 0;
            }
        }

    }

    fn draw_row(&mut self, x: u8, y: u8, row: u8) -> bool {

        for i in 0..8 {
            // set the value of the pixel at the address to the xor of itself and the sprite bit at that address.
            let x_idx = ((x+i) as usize) % DISPLAY_WIDTH;
            let y_idx = (y as usize) % DISPLAY_HEIGHT;
            self.pixels[x_idx][y_idx] = self.pixels[x_idx][y_idx] ^ (row & 1 << (7-i));
        }

        // Returns 1 if a pixel was erased, 0 if no pixels were erased.
        return (x & y < 0);

    }

    fn draw_sprite(&mut self, x: u8, y: u8, sprite_vec: Vec<u8>) -> u8 {
        
        let mut pixel_erased = 0;

        let mut y_offset = 0;
        
        for sprite_row in sprite_vec {

            if (self.draw_row(x, y + y_offset, sprite_row)) {
                pixel_erased = 1;
            }
            y_offset += 1;
            
        }



        return pixel_erased;



    }



}




struct CPU {
    stack: Vec<u16>,
    I: u16,
    pc: u16,
    VX: Vec<u8>
}

impl CPU {
    fn new() -> Self {
        CPU {
            stack: Vec::new(),
            I: 0,
            pc: 0,
            VX: vec![0; 15]
        }
    }
}


struct Chip8 {
    display: Display,
    memory: Vec<u8>,
    cpu: CPU,
}

impl Chip8 {
    fn new() -> Self {
        Chip8 { display: Display{pixels: [[0; DISPLAY_WIDTH]; DISPLAY_HEIGHT]}, 
                memory: vec![0; MEM_SIZE],
                cpu: CPU::new()}
    }
    fn load_program(&mut self, path: path::PathBuf) {

        let contents = fs::read(path)
            .expect("Could not read file.");
    
        if contents.len() > MAX_PROGRAM_SIZE {
            // return error.
        }
    
        let mut file_it = contents.into_iter().peekable();
    
        let mut write_ptr = PROG_START;
    
        while file_it.peek().is_some() {
            
            self.memory[write_ptr as usize] = file_it.next().unwrap();
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
    
        vec_emplace_at(font, 0x050, &mut self.memory)
    
    }

    fn get_instruction(&mut self) -> Instruction {

        // get the two bytes at PC and convert them to the instruction type.

        let mut inst: u16;

        inst = (self.memory[self.cpu.pc as usize] << 0xFF) as u16;
        self.cpu.pc += 1;
        inst += self.memory[self.cpu.pc as usize] as u16;
        self.cpu.pc += 1;

        return Instruction::new(inst);

    }   

    fn clear_screen(&mut self) {
        self.display.clear();
    }
    
    fn jump(&mut self, address: u16) {
        
        // set the program counter to the 12 bit address provided.
        self.cpu.pc = address

    }

    fn set_vx(&mut self, register_idx: u8, val: u8) {

        self.cpu.VX[register_idx as usize] = val;

    }

    fn add_vx(&mut self, register_idx: u8, val: u8) {

        self.cpu.VX[register_idx as usize] += val;

    }

    fn set_index_register(&mut self, address: u16) {

        self.cpu.I = address;

    }


    fn draw(&mut self, x: u8, y: u8, n: u8) {

        // The interpreter reads n bytes from memory, starting at the address stored in I. 
        // These bytes are then displayed as sprites on screen at coordinates (Vx, Vy). Sprites are XORed onto the existing screen.
        // If this causes any pixels to be erased, VF is set to 1, otherwise it is set to 0.

        // Clear VF (does not need to be cleared in theory).
        self.cpu.VX[0xF] = 0;

        // Modulo operation wraps screen operations.
        let startX = self.cpu.VX[x as usize];
        let startY = self.cpu.VX[y as usize];

        let mut sprite: Vec<u8> = Vec::new();

        for i in (self.cpu.I)..(self.cpu.I + n as u16) {

            sprite.push(self.memory[i as usize]);

        }

        // Set VF
        self.cpu.VX[0xF] = self.display.draw_sprite(startX, startY, sprite);
    

    }


    fn run_instruction(&mut self, mut instruction: Instruction) {

        // Create nibbles from 16 bit instruction

        


        match instruction.get_nibble(0){
            0x0=>self.clear_screen(),
            0x1=>self.jump(instruction.instruction & 0xFFF),
            0x6=>self.set_vx(instruction.get_nibble(1), (instruction.instruction & 0xFF).try_into().unwrap()),
            0x7=>self.add_vx(instruction.get_nibble(1), (instruction.instruction & 0xFF).try_into().unwrap()),
            0xA=>self.set_index_register(instruction.instruction & 0xFFF),
            0xD=>self.draw(instruction.get_nibble(1), instruction.get_nibble(2), instruction.get_nibble(3)),
            _=>println!("Instruction not recognized!")
        }




    }


    fn run() {}
    




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






fn hex_dump(buf: Vec<u8>) {

    println!("{:x?}", buf);

}






fn main() {

    let mut chip8 = Chip8::new();


    //let mut memory: Vec<u8> = vec![0; MEM_SIZE];
    //let mut display: Display = Display{pixels: [[false; DISPLAY_WIDTH]; DISPLAY_HEIGHT]};

    //let mut rom_path = exec_path.clone();
    let mut rom_path = path::PathBuf::from(r"C:\Users\Nick\Documents\GitHub\RustCHIP8");
    rom_path.push("roms");
    rom_path.push(r"IBM Logo.ch8");

   
    chip8.load_program(rom_path);








    



    hex_dump(chip8.memory);

}
