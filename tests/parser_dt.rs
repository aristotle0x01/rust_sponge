use rust_sponge::util::buffer::Buffer;
use rust_sponge::util::parser::{NetParser, NetUnparser};

#[test]
fn t_parser_dt() {
    const VAL1: u32 = 0xdeadbeef;
    const VAL2: u16 = 0xc0c0;
    const VAL3: u8 = 0xff;
    const VAL4: u32 = 0x0c05fefe;

    // first, let's serialize it
    let mut buffer: Vec<u8> = Vec::new();
    buffer.push(0x32);
    {
        NetUnparser::u32(&mut buffer, VAL1);
        NetUnparser::u16(&mut buffer, VAL2);
        NetUnparser::u8(&mut buffer, VAL3);
        NetUnparser::u32(&mut buffer, VAL4);
    } // p goes out of scope, data is in buffer

    // now let's deserialize it
    let out0: u8;
    let out3: u8;
    let out1: u32;
    let out4: u32;
    let out2: u16;
    {
        let mut b = Buffer::new(buffer);
        let mut p = NetParser::new(&mut b); // NOTE: starting at offset 0
        out0 = p.u8(); // buffer[0], which we manually set to 0x32 above
        out1 = p.u32(); // parse out VAL1
        out2 = p.u16(); // VAL2
        out3 = p.u8(); // VAL3
        out4 = p.u32(); // VAL4
    } // p goes out of scope

    if out0 != 0x32 || out1 != VAL1 || out2 != VAL2 || out3 != VAL3 || out4 != VAL4 {
        panic!("bad parse");
    }
}
