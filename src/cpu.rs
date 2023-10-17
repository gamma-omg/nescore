use crate::memory::Memory;
use self::addressing::{AddressMode, Value};

mod addressing
{
    use super::CPU;

    pub enum Value
    {
        Invalid,
        FromAccumulator,
        AtAddress(u16)
    }

    impl Value
    {
        pub fn get(&self, cpu: &CPU) -> u8 
        {
            match self {
                Value::FromAccumulator => cpu.registers.A,
                Value::AtAddress(addr) => cpu.memory.read8(*addr),
                Value::Invalid => panic!("Unable to read: location is invalid")
            }
        }

        pub fn set(&self, cpu: &mut CPU, val: u8)
        {
            match self {
                Value::FromAccumulator => cpu.registers.A = val,
                Value::AtAddress(addr) => cpu.memory.write8(*addr, val),
                Value::Invalid => panic!("Unable to write: location is invalid")
            }
        }
    }

    pub struct AccessResult
    {
        pub value: Value,
        pub cycles: u8,
        pub pc_offset: u16
    }    

    pub enum AddressMode
    {
        None,
        Acc,
        Imm,
        Zp,
        Zpx
    }
    
    impl AddressMode
    {
        pub fn read(&self, cpu: &mut CPU) -> AccessResult
        {
            match self {
                AddressMode::None => {
                    AccessResult {
                        value: Value::Invalid,
                        cycles: 1,
                        pc_offset: 0
                    }
                }
                AddressMode::Acc => {
                    AccessResult {
                        value: Value::FromAccumulator,
                        cycles: 2,
                        pc_offset: 0
                    }
                },
                AddressMode::Imm => {
                    AccessResult {
                        value: Value::AtAddress(cpu.registers.PC),
                        cycles: 2,
                        pc_offset: 1
                    }
                }
                AddressMode::Zp => {
                    AccessResult {
                        value: Value::AtAddress(cpu.memory.read8(cpu.registers.PC) as u16),
                        cycles: 3,
                        pc_offset: 1
                    }
                },
                AddressMode::Zpx => {
                    AccessResult {
                        value: Value::AtAddress(cpu.memory.read8(cpu.registers.PC) as u16 + cpu.registers.X as u16),
                        cycles: 4,
                        pc_offset: 1
                    }
                }
            }    
        }
    }
    
}

type OpImpl = fn(&mut CPU, operand: &mut Value);

pub struct Op
{
    op_impl: OpImpl,
    operand: Value,
    cycle: u8,
    total_cycles: u8,
}

impl Op
{
    fn new(cpu: &mut CPU, op_impl: OpImpl, addr_mode: AddressMode) -> Op
    {
        let result = addr_mode.read(cpu);
        cpu.registers.PC += result.pc_offset;

        Op {
            op_impl: op_impl,
            operand: result.value,
            total_cycles: result.cycles,
            cycle: 0
        }
    }

    fn tick(&mut self, cpu: &mut CPU) -> bool
    {
        self.cycle += 1;
        if self.cycle < self.total_cycles {
            return false;
        }

        (self.op_impl)(cpu, &mut self.operand);
        true
    }
}

enum StatusFlags
{
    C = 0b00000001,
    Z = 0b00000010,
    I = 0b00000100,
    D = 0b00001000,
    B = 0b00010000,
   _1 = 0b00100000,
    V = 0b01000000,
    N = 0b10000000
}

struct Registers
{
    PC: u16,
    SP: u8,
    A : u8,
    X : u8,
    Y : u8,
    PS: u8,
}

impl Registers
{
    #[inline(always)]
    fn get_flag(&self, flag: StatusFlags) -> bool
    {
        self.PS & flag as u8 > 0
    }

    #[inline(always)]
    fn set_flag(&mut self, flag: StatusFlags, is_set:bool)
    {
        if is_set {
            self.PS = self.PS | flag as u8;
        }
        else {
            self.PS = self.PS & !(flag as u8);
        }
    }
}

pub struct CPU
{
    memory: Box<Memory>,
    registers: Registers,
    cycle: usize,
    op: Option<Op>
}

impl CPU
{
    pub fn new(memory: Box<Memory>) -> CPU
    {
        CPU {
            memory: memory,
            registers: Registers {
                PC: 0,
                SP: 0,
                A : 0,
                X : 0,
                Y : 0,
                PS: 0,
            },
            cycle: 0,
            op: None
        }
    } 

    pub fn tick(&mut self)
    {
        if self.op.is_none() {
            self.op = Some(self.read_op());
        }

        let cur_op = self.op.take();
        let mut op = cur_op.unwrap();
        if op.tick(self) {
            self.op = None
        }
        else {
            self.op = Some(op);
        }

        self.cycle += 1;
    }

    pub fn ticks(&mut self, n:usize) 
    {
        for _ in 0..n {
            self.tick();
        }
    }

    fn read_op(&mut self) -> Op
    {
        let op_code = self.memory.read8(self.registers.PC);
        self.registers.PC += 1;
        let op_factory = instructions::OPCODE_MAP[op_code as usize];
        op_factory(self)
    }
}

mod instructions
{
    use super::{CPU, StatusFlags, addressing::{AddressMode, Value}, Op};

    pub const OPCODE_MAP: [fn(&mut CPU) -> Op; 0x100] = [
      //       0       1       2       3       4       5       6       7       8       9       A       B       C       D       E       F
      /* 0 */ nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,
      /* 1 */ nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,
      /* 2 */ nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,
      /* 3 */ nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,
      /* 4 */ nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,
      /* 5 */ nop,    nop,    nop,    nop,    nop,    nop,  adc_zp, adc_zpx,  nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,
      /* 6 */ nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,  adc_imm,  nop,    nop,    nop,    nop,    nop,    nop,
      /* 7 */ nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,
      /* 8 */ nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,
      /* 9 */ nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,
      /* A */ nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,
      /* B */ nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,
      /* C */ nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,
      /* D */ nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,
      /* E */ nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,
      /* F */ nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,    nop,        
    ];

    fn nop(cpu: &mut CPU) -> Op
    {
        Op::new(cpu, |_, _|{}, AddressMode::None)
    }

    fn adc_imm(cpu: &mut CPU) -> Op
    {      
        Op::new(cpu, _adc, AddressMode::Imm)
    }

    fn adc_zp(cpu: &mut CPU) -> Op
    {        
        Op::new(cpu, _adc, AddressMode::Zp)
    }

    fn adc_zpx(cpu: &mut CPU) -> Op
    {        
        Op::new(cpu, _adc, AddressMode::Zpx)
    }

    fn _adc(cpu: &mut CPU, arg: &mut Value)
    {
        let base = cpu.registers.A as u16;
        let operand = arg.get(cpu) as u16;
        let mut result: u16 = base + operand;
        if cpu.registers.get_flag(StatusFlags::C) {
            result += 1;
        }        

        cpu.registers.set_flag(StatusFlags::C, result > 0xFF);
        
        let operand = operand as u8;
        let result: u8 = result as u8;
        cpu.registers.set_flag(StatusFlags::Z, result == 0);
        cpu.registers.set_flag(StatusFlags::V, (operand ^ result) & (result ^ cpu.registers.A) & 0x80 != 0);
        cpu.registers.set_flag(StatusFlags::N, result & 0b10000000 > 0);

        cpu.registers.A = result as u8;
    }
}

#[cfg(test)]
mod tests
{
    use crate::memory::Memory;
    use super::CPU;

    fn from_program(program: Vec<u8>) -> CPU
    {
        let mem = Memory::from_buffer(program);
        CPU::new(Box::new(mem))
    }

    mod adc
    {
        use std::vec;

        use crate::cpu::{tests::from_program, StatusFlags};

        #[test]
        fn adc_imm()
        {
            let mut cpu = from_program(vec![0x69, 0x02]);
            cpu.ticks(2);
            assert_eq!(cpu.registers.A, 2);
        }

        #[test]
        fn adc_imm_multiple()
        {
            let mut cpu = from_program(vec![
                0x69, 0x02,
                0x69, 0x03,
                0x69, 0x04
            ]);

            cpu.ticks(6);

            assert_eq!(cpu.registers.A, 9);
        }

        #[test]
        fn adc_imm_z_flag_set()
        {
            let mut cpu = from_program(vec![
                0x69, 0xFF,
                0x69, 0x01
            ]);

            cpu.ticks(4);

            assert!(cpu.registers.get_flag(StatusFlags::Z));
        }

        #[test]
        fn adc_imm_z_flag_unset()
        {
            let mut cpu = from_program(vec![
                0x69, 0x01,
                0x69, 0x02
            ]);

            cpu.ticks(4);

            assert!(!cpu.registers.get_flag(StatusFlags::Z));
        }

        #[test]
        fn adc_imm_v_flag_set()
        {
            let mut cpu = from_program(vec![
                0x69, 0x7F,
                0x69, 0x01
            ]);
  
            cpu.ticks(4);

            assert_eq!(cpu.registers.A as i8, -128);
            assert!(cpu.registers.get_flag(StatusFlags::V));
        }  

        #[test]
        fn adc_imm_v_flag_unset()
        {
            let mut cpu = from_program(vec![
                0x69, 0x02,
                0x69, 0x02
            ]);
  
            cpu.ticks(4);

            assert!(!cpu.registers.get_flag(StatusFlags::V));
        }  

        #[test]
        fn adc_imm_n_flag_set()
        {
            let mut cpu = from_program(vec![
                0x69, 0xF0,
                0x69, 0x02
            ]);
            
            cpu.ticks(4);

            assert!(cpu.registers.get_flag(StatusFlags::N));            
        }

        #[test]
        fn adc_imm_n_flag_unset()
        {
            let mut cpu = from_program(vec![
                0x69, 0x02,
                0x69, 0x02
            ]);

            cpu.ticks(4);

            assert!(!cpu.registers.get_flag(StatusFlags::N));
        }
    }    
}