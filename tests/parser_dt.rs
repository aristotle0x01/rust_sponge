use rust_sponge::util::buffer::Buffer;
use rust_sponge::util::parser::{NetParser, NetUnparser};

#[test]
fn t_parser_dt() {
    const val1: u32 = 0xdeadbeef;
    const val2: u16 = 0xc0c0;
    const val3: u8 = 0xff;
    const val4: u32 = 0x0c05fefe;

    // first, let's serialize it
    let mut buffer = String::with_capacity(0);
    buffer.push(0x32 as char);
    {
        NetUnparser::u32(&mut buffer, val1);
        NetUnparser::u16(&mut buffer, val2);
        NetUnparser::u8(&mut buffer, val3);
        NetUnparser::u32(&mut buffer, val4);
    }  // p goes out of scope, data is in buffer

    // now let's deserialize it
    let mut out0: u8;
    let mut out3: u8;
    let mut out1: u32;
    let mut out4: u32;
    let mut out2: u16;
    {
        let mut p = NetParser::new(Buffer::new(buffer)); // NOTE: starting at offset 0
        out0 = p.u8();                     // buffer[0], which we manually set to 0x32 above
        out1 = p.u32();                    // parse out val1
        out2 = p.u16();                    // val2
        out3 = p.u8();                     // val3
        out4 = p.u32();                    // val4
    }  // p goes out of scope

    if out0 != 0x32 || out1 != val1 || out2 != val2 || out3 != val3 || out4 != val4 {
        panic!("bad parse");
    }
}
