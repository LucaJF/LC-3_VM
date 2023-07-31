/// 以下实现规则可以从assets/lc3-isa.pdf中找到
use lc_3_vm::register::Reg;
use lc_3_vm::{mem_read, mem_write, sign_extend, update_flags};

/// 注意：将传递到我们的模拟器的汇编代码
/// 严重依赖整数溢出加法来进行环绕。
/// Rust 不允许在正常添加中这样做，例如：let a: u16 = 65535 + 1
/// 会产生错误。 为此，我们使用了 u16::wrapping_add() 函数。
/// u16::wrapping_add(65536, 1) 与 65535 + 1 相同，在这种情况下产生 0。

/// Add
pub fn op_add(reg: &mut Vec<u16>, instr: u16) {
    // 以下的into()是用于从u16转换成usize的
    let r0: usize = ((instr >> 9) & 0x07).into(); // DR(destination register)
    let r1: usize = ((instr >> 6) & 0x07).into(); // SR1(getting first operand register)
    let imm_flag = (instr >> 5) & 0x01; // immediate mode 0/1

    if imm_flag == 1 {
        // 先获取Immediate number然后再处理一下正负
        let imm5: u16 = sign_extend(instr & 0x1f, 5);
        reg[r0] = u16::wrapping_add(reg[r1], imm5);
    } else {
        let r2: usize = (instr & 0x07).into(); // SR2(last 3 bits of instruction is second operand)
        reg[r0] = u16::wrapping_add(reg[r1], reg[r2]);
    }

    update_flags(r0, reg);
}

/// //Bitwise And
pub fn op_and(reg: &mut Vec<u16>, instr: u16) {
    let r0: usize = ((instr >> 9) & 0x07).into();
    let r1: usize = ((instr >> 6) & 0x07).into();
    let imm_flag: u16 = (instr >> 5) & 0x01;

    if imm_flag == 1 {
        let imm5: u16 = sign_extend(instr & 0x1f, 5);
        reg[r0] = reg[r1] & imm5;
    } else {
        let r2: usize = (instr & 0x7).into();
        reg[r0] = reg[r1] & reg[r2];
    }

    update_flags(r0, reg);
}

/// Bitwise Not
pub fn op_not(reg: &mut Vec<u16>, instr: u16) {
    let r0: usize = ((instr >> 9) & 0x07).into();
    let r1: usize = ((instr >> 6) & 0x07).into();

    reg[r0] = !reg[r1];
    update_flags(r0, reg);
}

/// Branch
pub fn op_branch(reg: &mut Vec<u16>, instr: u16) {
    let pc_offset: u16 = sign_extend(instr & 0x1FF, 9);
    let cond_flag: u16 = (instr >> 9) & 0x07;

    if (cond_flag & reg[Reg::COND]) > 0 {
        reg[Reg::PC] = u16::wrapping_add(reg[Reg::PC], pc_offset);
    }
}

/// Note: RET is actually just a special case of JUMP
pub fn op_jump(reg: &mut Vec<u16>, instr: u16) {
    let r1: usize = ((instr >> 6) & 0x07).into();

    reg[Reg::PC] = reg[r1];
}

/// Jump Register
pub fn op_jsr(reg: &mut Vec<u16>, instr: u16) {
    let long_flag: u16 = (instr >> 11) & 1;
    reg[Reg::R7] = reg[Reg::PC];

    if long_flag == 1 {
        let long_pc_offset = sign_extend(instr & 0x7FF, 11);
        reg[Reg::PC] = u16::wrapping_add(reg[Reg::PC], long_pc_offset);
    } else {
        let r1: usize = ((instr >> 6) & 0x07).into();
        reg[Reg::PC] = reg[r1];
    }
}

/// "Load - An address is computed by sign-extending bits [8:0]
/// to 16 bits and adding this value to the incremented PC. The
/// contents of memory at this address are loaded into DR. The
/// condition codes are set, based on whether the value loaded
/// is negative, zero, or positive."
pub fn op_load(reg: &mut Vec<u16>, instr: u16, memory: &mut Vec<u16>) {
    let r0: usize = ((instr >> 9) & 0x7).into();
    let pc_offset: u16 = sign_extend(instr & 0x1FF, 9);

    reg[r0] = mem_read(u16::wrapping_add(reg[Reg::PC], pc_offset), memory);
    update_flags(r0, reg);
}

/// Load Indirect - Load a value from a location in memory into register
pub fn op_ldi(reg: &mut Vec<u16>, instr: u16, memory: &mut Vec<u16>) {
    let r0: usize = ((instr >> 9) & 0x07).into();
    let pc_offset: u16 = sign_extend(instr & 0x1FF, 9);

    reg[r0] = mem_read(
        mem_read(u16::wrapping_add(reg[Reg::PC], pc_offset), memory),
        memory,
    );
    update_flags(r0, reg);
}

/// "Load Register - An address is computed by sign-extending bits
/// [5:0] to 16 bits and adding this value to the contents of the
/// register specified by bits [8:6]. The contents of memory at
/// this address are loaded into DR.
pub fn op_ldr(reg: &mut Vec<u16>, instr: u16, memory: &mut Vec<u16>) {
    let r0: usize = ((instr >> 9) & 0x7).into();
    let r1: usize = ((instr >> 6) & 0x7).into();
    let offset: u16 = sign_extend(instr & 0x3F, 6);

    reg[r0] = mem_read(u16::wrapping_add(reg[r1], offset), memory);
    update_flags(r0, reg);
}

/// "Load Effective Address - An address is computed by sign-extending
/// bits [8:0] to 16 bits and adding this value to the incremented PC.
/// This address is loaded into DR."
pub fn op_lea(reg: &mut Vec<u16>, instr: u16) {
    let r0: usize = ((instr >> 9) & 0x07).into();
    let pc_offset: u16 = sign_extend(instr & 0x1FF, 9);

    reg[r0] = u16::wrapping_add(reg[Reg::PC], pc_offset);
    update_flags(r0, reg);
}

/// "Store - The contents of the register specified by SR are stored
/// in the memory location whose address is computed by sign-extending
/// bits [8:0] to 16 bits and adding this value to the incremented PC."
pub fn op_st(reg: &mut Vec<u16>, instr: u16, memory: &mut Vec<u16>) {
    let r0: usize = ((instr >> 9) & 0x07).into();
    let pc_offset: u16 = sign_extend(instr & 0x1FF, 9);

    mem_write(u16::wrapping_add(reg[Reg::PC], pc_offset), reg[r0], memory);
}

/// "Store Indirect Address - The contents of the register specified
/// by SR are stored in the memory location whose address is obtained as
/// follows: Bits [8:0] are sign-extended to 16 bits and added to the
/// incremented PC. What is in memory at this address is the address of
/// the location to which the data in SR is stored."
pub fn op_sti(reg: &mut Vec<u16>, instr: u16, memory: &mut Vec<u16>) {
    let r0: usize = ((instr >> 9) & 0x07).into();
    let pc_offset: u16 = sign_extend(instr & 0x1FF, 9);

    mem_write(
        mem_read(u16::wrapping_add(reg[Reg::PC], pc_offset), memory),
        reg[r0],
        memory,
    );
}

/// "Store Register - The contents of the register specified by SR
/// are stored in the memory location whose address is computed by
/// sign-extending bits [5:0] to 16 bits and adding this value to
/// the contents of the register specified by bits [8:6]."
pub fn op_str(reg: &mut Vec<u16>, instr: u16, memory: &mut Vec<u16>) {
    let r0: usize = ((instr >> 9) & 0x07).into();
    let r1: usize = ((instr >> 6) & 0x07).into();
    let offset: u16 = sign_extend(instr & 0x3F, 6);

    mem_write(u16::wrapping_add(reg[r1], offset), reg[r0], memory);
}
