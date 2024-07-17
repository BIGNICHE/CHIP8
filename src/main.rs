use std::{env, fs, io, ops::BitOr, panic::PanicInfo, path, time::{Duration, Instant}};
static MEM_SIZE: usize = 0x1000; // Size of CHIP-8 Memory.
static PROG_START: i32 = 0x200;
static MAX_PROGRAM_SIZE: usize = 0xE00;
static FONT_MEMORY_START: i32 = 0x50;
const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
use error_iter::ErrorIter as _;
use log::{debug, error};
use pixels::{Error, Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;


struct Display {
    pixels: [[u8; DISPLAY_HEIGHT]; DISPLAY_WIDTH],
}

struct RegisterOperator<'a> {
    VX: &'a mut [u8],
}   

impl RegisterOperator<'_> {

    fn ld(&mut self, x_idx: usize, y_idx: usize) {
        self.VX[x_idx] = self.VX[y_idx];
    }

    fn bit_or(&mut self, x_idx: usize, y_idx: usize) {
        self.VX[x_idx] | self.VX[y_idx];
    }
    fn bit_and(&mut self, x_idx: usize, y_idx: usize) {
        self.VX[x_idx] & self.VX[y_idx];
    }
    fn bit_xor(&mut self, x_idx: usize, y_idx: usize) {
        (self.VX[x_idx]) ^ self.VX[y_idx];
    }
    fn add(&mut self, x_idx: usize, y_idx: usize) {
        // VF is carry.
        // If the result is greater than 8 bits, VF is set to 1.
        // only the lowest 8 bits are kept.
        let tmp: u16 = ((self.VX[x_idx]) + self.VX[y_idx]) as u16;
        if tmp > 0xFF {
            self.VX[0xF] = 1;
        } else {
            self.VX[0xF] = 0;
        }
        (self.VX[x_idx]) = (tmp & 0xFF) as u8;
    }

    fn sub(&mut self, x_idx: usize, y_idx: usize) {

        // Set VF as a carry indicator

        if self.VX[x_idx] > self.VX[y_idx] {
            self.VX[0xF] = 1;
        } else {
            self.VX[0xF] = 0;
        }

        // Unsigned sub, so result wraps.
        self.VX[x_idx] = self.VX[x_idx] - self.VX[y_idx];
        
    }

    fn subn(&mut self, x_idx: usize, y_idx: usize) {
        if self.VX[x_idx] > self.VX[y_idx] {
            self.VX[0xF as usize] = 0;
        } else {
            self.VX[0xF as usize] = 1;
        }
        // Unsigned sub, so result wraps.
        self.VX[x_idx] = self.VX[x_idx] - self.VX[y_idx];

    }

    // shift right
    fn shr(&mut self, x_idx: usize, y_idx: usize) {

        self.VX[0xF as usize] = self.VX[x_idx] & 1;

        self.VX[x_idx] >> 1;



    }

    fn shl(&mut self, x_idx: usize, y_idx: usize) {
        // VF is set to the MSB of VX
        self.VX[0xF as usize] = (self.VX[x_idx] & 0x80) >> 7;

        self.VX[x_idx] << 1;
    }

    


}






struct Timer {
    val: u8,
    next_tick: Instant
}

impl Timer {
    fn new() -> Self {
        Timer{val: 0, next_tick: Instant::now() }
    }
    fn update(&mut self) {

        let current_time = Instant::now();

        if self.next_tick < current_time {

            self.next_tick = current_time + Duration::from_secs(1);

            if (self.val > 0) {
                self.val -= 1;
            }

        }
    }
}

struct CPU {
    stack: Vec<u16>,
    I: u16,
    pc: u16,
    VX: Vec<u8>,
    delay_timer: Timer,
    sound_timer: Timer
}

struct Chip8 {
    display: Display,
    memory: Vec<u8>,
    cpu: CPU,
}

fn get_nibble(instruction: &u16, nibble_index: &u8) -> u8 {
    // Panics if value is not a u8. Since it is being logcal anded with 1111, it should fit within 4 bits.
    let nibble: u8 = ((instruction >> 4 * (3 - nibble_index)) & 0xF)
        .try_into()
        .unwrap();

    return nibble;
}

fn emplace_at(data: u8, emplace_idx: usize, memory: &mut Vec<u8>) {
    memory[emplace_idx] = data;
}

fn vec_emplace_at(data: Vec<u8>, emplace_idx: usize, memory: &mut Vec<u8>) {
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

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();

    let window = {
        let size = LogicalSize::new(DISPLAY_WIDTH as f64, DISPLAY_HEIGHT as f64);
        let scaled_size = LogicalSize::new(DISPLAY_WIDTH as f64 * 3.0, DISPLAY_HEIGHT as f64 * 3.0);
        WindowBuilder::new()
            .with_title("CHIP-8")
            .with_inner_size(scaled_size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(
            DISPLAY_WIDTH.try_into().unwrap(),
            DISPLAY_HEIGHT.try_into().unwrap(),
            surface_texture,
        )?
    };

    let mut chip8 = Chip8::new();

    //let mut memory: Vec<u8> = vec![0; MEM_SIZE];
    //let mut display: Display = Display{pixels: [[false; DISPLAY_WIDTH]; DISPLAY_HEIGHT]};

    //let mut rom_path = exec_path.clone();
    let mut rom_path = path::PathBuf::from(r"C:\Users\Vishu\Documents\GitHub\CHIP8");
    rom_path.push("roms");
    rom_path.push(r"ibm.ch8");

    chip8.load_program(rom_path);

    // chip8.run();

    // chip8.display.pixels[0][0] = 1;

    // chip8.display.print();

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            chip8.display.update_display(pixels.frame_mut());
            if let Err(err) = pixels.render() {
                log_error("pixels.render", err);
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.close_requested() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    log_error("pixels.resize_surface", err);
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }

            // Update internal state and request a redraw
            chip8.run();
            window.request_redraw();
        }
    });

    //hex_dump(chip8.memory);
}

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}

impl Chip8 {
    fn new() -> Self {
        Chip8 {
            display: Display {
                pixels: [[0; DISPLAY_HEIGHT]; DISPLAY_WIDTH],
            },
            memory: vec![0; MEM_SIZE],
            cpu: CPU::new(),
        }
    }
    fn load_program(&mut self, path: path::PathBuf) {
        let contents = fs::read(path).expect("Could not read file.");

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
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];

        vec_emplace_at(font, 0x050, &mut self.memory)
    }

    fn get_instruction(&mut self) -> u16 {
        // get the two bytes at PC and convert them to the instruction type.

        let mut inst: u16;

        inst = (self.memory[self.cpu.pc as usize] as u16) << 8;
        self.cpu.pc += 1;
        inst += self.memory[self.cpu.pc as usize] as u16;
        self.cpu.pc += 1;

        return inst;
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
        // TAKEN FROM COWGOD'S CHIP 8 TECHNICAL REFERENCE

        // The interpreter reads n bytes from memory, starting at the address stored in I.
        // These bytes are then displayed as sprites on screen at coordinates (Vx, Vy). Sprites are XORed onto the existing screen.
        // If this causes any pixels to be erased, VF is set to 1, otherwise it is set to 0.

        // Clear VF 
        self.cpu.VX[0xF] = 0;

        // Modulo operation wraps screen operations.
        let start_x = self.cpu.VX[x as usize];
        let start_y = self.cpu.VX[y as usize];

        let mut sprite: Vec<u8> = Vec::new();

        for i in (self.cpu.I)..(self.cpu.I + n as u16) {
            sprite.push(self.memory[i as usize]);
        }

        // Set VF
        self.cpu.VX[0xF] = self.display.draw_sprite(start_x, start_y, sprite);

        /*

        unused code to print to console directly

        self.display.print();

        println!("");
        println!("");
        println!("");
        println!("");
        println!("");
        */
    }

    // Not implemented on modern interpreters
    fn ret(&mut self){}


    // Not implemented on modern interpreters
    fn SYS(&mut self, address: u16){}

    fn decode_zero_instruction(&mut self, instruction: &u16) {

        match instruction {
            0x00E0 => self.clear_screen(),
            0x00EE => self.ret(),

            _=>self.SYS(instruction & 0xFFF)
        }

    }

    // Place the program counter on the stack, then replace it with the provided address
    fn call(&mut self, address: u16) {
        self.cpu.stack.push(self.cpu.pc);
        self.cpu.pc = address;
    }

    // Increments the program counter by 2 if Vn = val
    fn skip_equal(&mut self, instruction: &u16) {

        let register_idx = get_nibble(instruction, &1);
        let val = (instruction & 0xFF) as u8;

        if self.cpu.VX[register_idx as usize] == val {
            self.cpu.pc += 2
        }
    }

    fn skip_not_equal(&mut self, instruction: &u16) {

        let register_idx = get_nibble(instruction, &1);
        let val = (instruction & 0xFF) as u8;

        if self.cpu.VX[register_idx as usize] != val {
            self.cpu.pc += 2
        }

    }


    // 5xy0
    // Skip next instruction if Vx = Vy.
    fn skip_registers_equal(&mut self, instruction: &u16) {

        let r1 = get_nibble(instruction, &1);
        let r2 = get_nibble(instruction, &2);

        if self.cpu.VX[r1 as usize] == self.cpu.VX[r2 as usize] {
            self.cpu.pc += 2
        }

    }

    /*

    // 8xy2
    // Stores the bitwise AND of vx and vy in vx.
    fn and_x_y(&mut self, vx_index: usize, vy_index: usize) {

        self.cpu.VX[vx_index] = self.cpu.VX[vx_index] & self.cpu.VX[vy_index]; 

    }

    // 8xy3
    // Stores the bitwise OR of vx and vy in vx.
    fn and_x_y(&mut self, vx_index: usize, vy_index: usize) {

        self.cpu.VX[vx_index] = self.cpu.VX[vx_index] & self.cpu.VX[vy_index]; 

    }

     */

    fn decode_8_instruction(&mut self, instruction: &u16) {

        // Instructions beginning with 8 perform operations on the registers.
        // They are differentiated by the lowest significant nibble (index 3)
        let vx_index = get_nibble(instruction, &1) as usize;
        let vy_index = get_nibble(instruction, &2) as usize;

        let mut op = RegisterOperator{ VX: &mut self.cpu.VX };

        match get_nibble(instruction, &3) {
            0x0 => op.ld(vx_index, vy_index),
            0x1 => op.bit_or(vx_index, vy_index),
            0x2 => op.bit_and(vx_index, vy_index),
            0x3 => op.bit_xor(vx_index, vy_index),
            0x4 => op.add(vx_index, vy_index),
            0x5 => op.sub(vx_index, vy_index),
            0x6 => op.shr(vx_index, vy_index),
            0x7 => op.subn(vx_index, vy_index),
            0xE => op.shl(vx_index, vy_index),
            _ => println!("Instruction not recognized!")
        }
    }

    fn run_instruction(&mut self, instruction: &u16) {
        

        match (get_nibble(instruction, &3)) {
            0x0 => self.decode_zero_instruction(instruction),
            0x1 => self.jump(instruction & 0xFFF),
            0x2 => self.call(instruction & 0xFFF),
            0x3 => self.skip_equal(instruction),
            0x4 => self.skip_not_equal(instruction),
            0x5 => self.skip_registers_equal(instruction),
            0x6 => self.set_vx(
                get_nibble(instruction, &1),
                (instruction & 0xFF).try_into().unwrap(),
            ),
            0x7 => self.add_vx(
                get_nibble(instruction, &1),
                (instruction & 0xFF).try_into().unwrap(),
            ),
            0x8 => self.decode_8_instruction(instruction),
            0xA => self.set_index_register(instruction & 0xFFF),
            0xD => self.draw(
                get_nibble(instruction, &1),
                get_nibble(instruction, &2),
                get_nibble(instruction, &3),
            ),
            _ => println!("Instruction not recognized!"),
        }
    }

    fn run(&mut self) {
        self.cpu.delay_timer.update();
        self.cpu.sound_timer.update();
        let inst = self.get_instruction();
        self.run_instruction(&inst);
    }
}

impl CPU {
    fn new() -> Self {
        CPU {
            stack: Vec::new(),
            I: 0,
            pc: PROG_START as u16,
            VX: vec![0; 16],
            delay_timer: Timer::new(),
            sound_timer: Timer::new(),
            
        }
    }
}

impl Display {
    fn update_display(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let y: usize = ((i / DISPLAY_WIDTH) as f64).floor() as usize;
            let x: usize = (i % DISPLAY_WIDTH);

            let rgba = if self.pixels[x][y] > 0 {
                [0xFF, 0xFF, 0xFF, 0xFF]
            } else {
                [0x00, 0x00, 0x00, 0xFF]
            };

            pixel.copy_from_slice(&rgba);
        }
    }

    fn print_to_console(&mut self) {
        // draws to console
        for j in 0..DISPLAY_HEIGHT {
            for i in 0..DISPLAY_WIDTH {
                if self.pixels[i][j] > 0 {
                    print!("\u{2B1C}");
                } else {
                    print!("\u{2B1B}");
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
            let x_idx = ((x + i) as usize) % DISPLAY_WIDTH;
            let y_idx = (y as usize) % DISPLAY_HEIGHT;
            self.pixels[x_idx][y_idx] = self.pixels[x_idx][y_idx] ^ (row & 1 << (7 - i));
        }

        // Returns 1 if a pixel was erased, 0 if no pixels were erased.
        return ((x & y) > 0);
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