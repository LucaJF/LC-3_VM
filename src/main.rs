extern crate termios;

use std::{env, process};
use termios::*;

use lc_3_vm::opcodes::OpCodes;
use lc_3_vm::register::Reg;
use lc_3_vm::{mem_read, read_image, TrapCode};

mod opcode;
use opcode::*;

mod trapcode;
use trapcode::*;

fn main() {
    // 获取输入参数
    let args = env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        println!("Error: 至少提供一个VM镜像地址");
        println!("Usage: lc-3_vm <image-file1> [image-file2]...");
        process::exit(2);
    }

    // 初始化VM内存
    // LC-3有65536个内存位置，每个位置能存16bits值
    // 所以一共内存有128KB
    let mut memory = vec![0u16; 65536];

    // 加载所有输入的镜像参数
    for i in 1..args.len() {
        if !read_image(&args[i], &mut memory) {
            println!("Failed to load image: {}", args[i]);
            process::exit(1);
        }
    }

    // 标准控制台的默认行为是从用户获取输入，并仅在输入换行符（按 Enter 按钮）时才处理它们。 为了玩游戏，需要更改终端的默认行为。
    // Platform Specifics (Unix here)
    // Setting terminal input/output behaviour such as accepting
    // character without the need for a newline character
    // Refer: https://stackoverflow.com/questions/26321592/how-can-i-read-one-character-from-stdin-without-having-to-hit-enter
    let stdin = 0;
    let termios = Termios::from_fd(stdin).unwrap();
    let mut new_termios = termios.clone(); // make a mutable copy of termios
                                           // that we will modify
    new_termios.c_iflag &= IGNBRK | BRKINT | PARMRK | ISTRIP | INLCR | IGNCR | ICRNL | IXON;
    new_termios.c_lflag &= !(ICANON | ECHO); // no echo and canonical mode
    tcsetattr(stdin, TCSANOW, &mut new_termios).unwrap();
    // Platform specific end

    // 初始化Register
    #[allow(non_snake_case)]
    let PC_START: u16 = 0x3000; // PC默认的起始位置
    let mut registers = vec![0u16; Reg::COUNT as usize];

    registers[Reg::PC] = PC_START;

    // 处理程序，步骤如下：
    // 1.从内存中的寄存器地址加载一条指令PC。
    // 2.增加PC寄存器。
    // 3.查看操作码以确定它应该执行哪种类型的指令。
    // 4.使用指令中的参数执行指令。
    // 5.返回步骤1。
    let mut running = true;

    while running {
        // 加载一条指令
        let instr = mem_read(registers[Reg::PC], &mut memory);

        // PC地址+1留待下次循环继续读取
        registers[Reg::PC] += 1;

        // 获取操作码
        let opcode = instr >> 12;
        //println!("Executing Instr {:#018b} and Opcode bit: {}", instr, opcode);

        // 开始匹配action
        match opcode {
            code if code == OpCodes::OP_ADD as u16 => {
                op_add(&mut registers, instr);
            }
            code if code == OpCodes::OP_AND as u16 => {
                op_and(&mut registers, instr);
            }
            code if code == OpCodes::OP_BR as u16 => {
                op_branch(&mut registers, instr);
            }
            code if code == OpCodes::OP_JMP as u16 => {
                op_jump(&mut registers, instr);
            }
            code if code == OpCodes::OP_JSR as u16 => {
                op_jsr(&mut registers, instr);
            }
            code if code == OpCodes::OP_LD as u16 => {
                op_load(&mut registers, instr, &mut memory);
            }
            code if code == OpCodes::OP_LDI as u16 => {
                op_ldi(&mut registers, instr, &mut memory);
            }
            code if code == OpCodes::OP_LDR as u16 => {
                op_ldr(&mut registers, instr, &mut memory);
            }
            code if code == OpCodes::OP_LEA as u16 => {
                op_lea(&mut registers, instr);
            }
            code if code == OpCodes::OP_NOT as u16 => {
                op_not(&mut registers, instr);
            }
            code if code == OpCodes::OP_ST as u16 => {
                op_st(&mut registers, instr, &mut memory);
            }
            code if code == OpCodes::OP_STI as u16 => {
                op_sti(&mut registers, instr, &mut memory);
            }
            code if code == OpCodes::OP_STR as u16 => {
                op_str(&mut registers, instr, &mut memory);
            }
            code if code == OpCodes::OP_RES as u16 => {
                println!("Bad OpCode 'RES' received. Aborting.");
                process::exit(10);
            }
            code if code == OpCodes::OP_RTI as u16 => {
                println!("Bad OpCode 'RTI' received. Aborting.");
                process::exit(10);
            }
            // 1111就是trap code
            code if code == OpCodes::OP_TRAP as u16 => {
                // 先处理最后8位以获取具体trapcode
                let trapcode = instr & 0xFF;
                // println!("Executing {} TRAP, Instr {:#018b}", trapcode, instr);

                match trapcode {
                    code if code == TrapCode::GETC as u16 => {
                        trap_getc(&mut registers);
                    }
                    code if code == TrapCode::OUT as u16 => {
                        trap_out(&mut registers);
                    }
                    code if code == TrapCode::PUTS as u16 => {
                        trap_puts(&mut registers, &mut memory);
                    }
                    code if code == TrapCode::IN as u16 => {
                        trap_in(&mut registers);
                    }
                    code if code == TrapCode::PUTSP as u16 => {
                        trap_putsp(&mut registers, &mut memory);
                    }
                    code if code == TrapCode::HALT as u16 => {
                        trap_halt();
                        running = false;
                    }
                    _ => {
                        println!("Invalid Trap Code received, aborting.");
                        process::exit(21);
                    }
                }
            }
            _ => {
                println!("Invalid Opcode received, aborting current image.");
                process::exit(20);
            }
        }
    }

    // reset the stdin to original termios data
    tcsetattr(stdin, TCSANOW, &termios).unwrap();

    println!("Shutting Down VM...");
}
