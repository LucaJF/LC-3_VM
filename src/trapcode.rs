/// 程序通常的开始位置在0x3000是因为更低的位置预留给了trap routine的代码了。
/// 它们实际上并未向 LC-3 引入任何新功能，它们只是提供了一种执行任务的便捷方法（类似于操作系统系统调用）。
/// 在官方的 LC-3 模拟器中，陷阱例程是用汇编语言编写的。当调用陷阱代码时，被PC移动到该代码的地址。
/// CPU 执行过程的指令，完成后，PC重置到初始调用后的位置。
use lc_3_vm::register::Reg;
use std::io::Read;

/// get character from keyboard, not echoed onto the terminal
pub fn trap_getc(reg: &mut Vec<u16>) {
    let mut buffer = [0 as u8; 1];
    std::io::stdin().read_exact(&mut buffer).unwrap();
    reg[Reg::R0] = buffer[0].into();
}

/// output a character
pub fn trap_out(reg: &mut Vec<u16>) {
    print!("{}", (reg[Reg::R0] as u8) as char);
}

/// output a word string
pub fn trap_puts(reg: &mut Vec<u16>, memory: &mut Vec<u16>) {
    let mut index = reg[Reg::R0] as usize;

    while index < memory.len() && memory[index] != 0 {
        print!("{}", (memory[index] as u8) as char);
        index = index + 1;
    }
}

/// get character from keyboard, echoed onto the terminal
pub fn trap_in(reg: &mut Vec<u16>) {
    print!("Enter a character: ");

    reg[Reg::R0] = std::io::stdin()
        .bytes()
        .next()
        .and_then(|result| result.ok())
        .map(|byte| byte as u16)
        .unwrap();
}

/// output a byte string
pub fn trap_putsp(reg: &mut Vec<u16>, memory: &mut Vec<u16>) {
    let mut index = reg[Reg::R0] as usize;

    while index < memory.len() && memory[index] != 0 {
        //A word in our VM is 16 bits
        let word: u16 = memory[index];

        //We get the two bytes from our word. bytes here is an array of u8
        let bytes = word.to_be_bytes();

        print!("{}", bytes[1] as char);

        if bytes[0] != 0 {
            print!("{}", bytes[0] as char);
        }

        index = index + 1;
    }
}

/// halt the program
pub fn trap_halt() {
    println!("HALT Trapcode received, Halting.");
}
