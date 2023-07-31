/// 提供基础结构和utility

pub mod register {
    use std::ops::{Index, IndexMut};

    /// 每个寄存器存16bits
    /// R0-R7是普通存储槽
    /// PC是程序计数器，它指向下一个要运行指令的内存地址
    /// COND是上一个指令计算完的结果标识有三种值：
    ///  FL_POS = 1 << 0, /* P */
    ///  FL_ZRO = 1 << 1, /* Z */
    ///  FL_NEG = 1 << 2, /* N */
    /// COUNT是当前计算机架构里寄存器的总数
    pub enum Reg {
        R0,
        R1,
        R2,
        R3,
        R4,
        R5,
        R6,
        R7,
        PC,
        COND,
        COUNT,
    }

    // 为了每次直接能用枚举替代索引访问Vec里的值实现了Index trait
    // 这样不用每次都Reg as usize
    impl<T> Index<Reg> for Vec<T> {
        type Output = T;

        fn index(&self, index: Reg) -> &Self::Output {
            &self[index as usize]
        }
    }

    impl<T> IndexMut<Reg> for Vec<T> {
        fn index_mut(&mut self, index: Reg) -> &mut Self::Output {
            &mut self[index as usize]
        }
    }
}

pub mod opcodes {
    use std::ops::{Index, IndexMut};

    #[allow(non_camel_case_types)]
    pub enum OpCodes {
        OP_BR,   // branch
        OP_ADD,  // add
        OP_LD,   // load
        OP_ST,   // store
        OP_JSR,  // jump register
        OP_AND,  // bitwise and
        OP_LDR,  // load register
        OP_STR,  // store register
        OP_RTI,  // unused
        OP_NOT,  // bitwise not
        OP_LDI,  // load indirect
        OP_STI,  // store indirect
        OP_JMP,  // jump
        OP_RES,  // reserved (unused)
        OP_LEA,  // load effective address
        OP_TRAP, // execute trap
    }

    impl<T> Index<OpCodes> for Vec<T> {
        type Output = T;

        fn index(&self, index: OpCodes) -> &Self::Output {
            &self[index as usize]
        }
    }

    impl<T> IndexMut<OpCodes> for Vec<T> {
        fn index_mut(&mut self, index: OpCodes) -> &mut Self::Output {
            &mut self[index as usize]
        }
    }
}

pub enum TrapCode {
    GETC = 0x20,  // 32 - get character from keyboard, not echoed onto the terminal
    OUT = 0x21,   // 33 - output a character
    PUTS = 0x22,  // 34 - output a word string
    IN = 0x23,    // 35 - get character from keyboard, echoed onto the terminal
    PUTSP = 0x24, // 36 - output a byte string
    HALT = 0x25,  // 37 - halt the program
}

/// 条件标志是寄存器里存储的上一次计算完成后的结果标记，只有三种值
#[allow(non_camel_case_types)]
pub enum CondFlags {
    FL_POS = 1 << 0, // Positive
    FL_ZRO = 1 << 1, // Zero
    FL_NEG = 1 << 2, // Negative
}

/// Memory Mapped Registers
/// 某些特殊寄存器无法从普通寄存器表访问。相反，在内存中为它们保留了一个特殊的地址。
/// 要读取和写入这些寄存器，您只需读取和写入它们的内存位置即可。这些称为内存映射寄存器。
/// 它们通常用于与特殊硬件设备交互。
/// LC-3 有两个需要实现的内存映射寄存器。它们是键盘状态寄存器（KBSR）和键盘数据寄存器（KBDR）。
/// 指示KBSR是否按下了某个键，并KBDR标识按下了哪个键。
#[allow(non_camel_case_types)]
pub enum MemMapReg {
    MR_KBSR = 0xFE00, //Keyboard Status Register. 0xFE00 = 65024.
    MR_KBDR = 0xFE02, //Keyboard Data Register. 0xFE02 = 65026.
}

use register::Reg;
use std::io::Read;
use std::{fs::File, path::Path};

/// 立即数模式值只有5位，但需要与16位数字相加。要进行加法，需要将这 5 位扩展为 16 位以匹配其他数字。
/// 对于正数，我们可以简单地在附加位中填充 0。对于负数，这会导致问题。例如，5 位中的 -1 是1 1111。
/// 如果我们只是用 0 来扩展它，则0000 0000 0001 1111等于 31。
/// 符号扩展通过为正数填充 0 和为负数填充 1 来纠正这个问题，从而保留原始值。
pub fn sign_extend(mut x: u16, bit_count: u16) -> u16 {
    //this checks if the last bit has a 1 (indicating negative number)
    if (x >> (bit_count - 1)) & 1 == 1 {
        //we extend the left side with 1's as it is a -ve number
        x |= 0xFFFF << bit_count;
    }
    x
}

/// 每当将值写入寄存器时，我们都需要更新标志以指示其符号。
pub fn update_flags(r: usize, reg: &mut Vec<u16>) {
    let val = reg[r];

    if val == 0 {
        reg[Reg::COND] = CondFlags::FL_ZRO as u16;
    } else if val >> 15 == 1 {
        /* a 1 in the left-most bit indicates negative */
        reg[Reg::COND] = CondFlags::FL_NEG as u16;
    } else {
        reg[Reg::COND] = CondFlags::FL_POS as u16;
    }
}

/// 将 LC-3 程序读入内存，比如obj目录下的文件，
/// 第一个16位是从内存中开始的地址，后面每16位都是一条指令
pub fn read_image(image: &str, memory: &mut Vec<u16>) -> bool {
    let path = Path::new(image);
    let mut file = File::open(path).expect("No such file exists.");

    let mut data = vec![];
    file.read_to_end(&mut data).expect("Buffer overflow.");

    // [[val0, val1], ...]
    let mut iter = data.chunks(2);

    // 第一个元素就是程序在内存中开始的地址，一般是0x3000 or 12288
    let pc = iter.next().unwrap();

    // data一个是u8，所以需要将两个字节组合成一个u16字，
    // 因为这就是我们的内存存储数据的方式。 也就是说，我们的内存的字长是16位。
    let mut pc = ((pc[0] as u16) << 8 | pc[1] as u16) as usize;

    for el in iter {
        memory[pc] = (el[0] as u16) << 8 | el[1] as u16;
        pc += 1;
    }

    true
}

/// 因为有Memory Mapped Registers的存在，所以读取内存时要先check是不是读取的KBSR
/// 是先处理一下值不是直接按addr返回
pub fn mem_read(addr: u16, memory: &mut Vec<u16>) -> u16 {
    if addr == MemMapReg::MR_KBSR as u16 {
        let mut buffer = [0; 1];
        std::io::stdin().read_exact(&mut buffer).unwrap();

        if buffer[0] != 0 {
            memory[MemMapReg::MR_KBSR as usize] = 1 << 15;
            memory[MemMapReg::MR_KBDR as usize] = buffer[0] as u16;
        } else {
            memory[MemMapReg::MR_KBSR as usize] = 0;
        }
    }

    memory[addr as usize]
}

/// 写入内存
pub fn mem_write(addr: u16, val: u16, memory: &mut Vec<u16>) {
    memory[addr as usize] = val;
}
