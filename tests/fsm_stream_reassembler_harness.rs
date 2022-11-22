use rust_sponge::stream_reassembler::StreamReassembler;
use rust_sponge::SizeT;

pub trait ReassemblerTestStep {
    fn execute(&self, bs: &mut StreamReassembler);
}

pub trait ReassemblerExpectation: ReassemblerTestStep {
    fn description(&self) -> String;
    fn to_string(&self) -> String {
        String::from(format!("Expectation: {}", self.description()))
    }
}

pub trait ReassemblerAction: ReassemblerTestStep {
    fn description(&self) -> String;
    fn to_string(&self) -> String {
        String::from(format!("Action: {}", self.description()))
    }
}

pub struct BytesAvailable {
    bytes: String,
}
impl ReassemblerTestStep for BytesAvailable {
    fn execute(&self, reassembler: &mut StreamReassembler) {
        println!("  step: {}", ReassemblerExpectation::to_string(self));

        assert_eq!(
            reassembler.stream_out().buffer_size(),
            self.bytes.len() as SizeT
        );
        let data = reassembler.stream_out_mut().read(self.bytes.len());
        assert_eq!(data, self.bytes);
    }
}
impl ReassemblerExpectation for BytesAvailable {
    fn description(&self) -> String {
        format!("stream_out().buffer_size() returned {}, and stream_out().read({}) returned the string \"{}\"", self.bytes.len(), self.bytes.len(), self.bytes)
    }
}
impl BytesAvailable {
    #[allow(dead_code)]
    pub fn new(_bytes: String) -> BytesAvailable {
        BytesAvailable { bytes: _bytes }
    }
}

pub struct BytesAssembled {
    bytes: SizeT,
}
impl ReassemblerTestStep for BytesAssembled {
    fn execute(&self, reassembler: &mut StreamReassembler) {
        println!("  step: {}", ReassemblerExpectation::to_string(self));
        assert_eq!(reassembler.stream_out().bytes_written(), self.bytes)
    }
}
impl ReassemblerExpectation for BytesAssembled {
    fn description(&self) -> String {
        format!("net bytes assembled = {}", self.bytes)
    }
}
impl BytesAssembled {
    #[allow(dead_code)]
    pub fn new(_bytes: SizeT) -> BytesAssembled {
        BytesAssembled { bytes: _bytes }
    }
}

pub struct UnassembledBytes {
    bytes: SizeT,
}
impl ReassemblerTestStep for UnassembledBytes {
    fn execute(&self, reassembler: &mut StreamReassembler) {
        println!("  step: {}", ReassemblerExpectation::to_string(self));
        assert_eq!(reassembler.unassembled_bytes(), self.bytes)
    }
}
impl ReassemblerExpectation for UnassembledBytes {
    fn description(&self) -> String {
        format!("bytes not assembled = {}", self.bytes)
    }
}
impl UnassembledBytes {
    #[allow(dead_code)]
    pub fn new(_bytes: SizeT) -> UnassembledBytes {
        UnassembledBytes { bytes: _bytes }
    }
}

pub struct AtEof;
impl ReassemblerTestStep for AtEof {
    fn execute(&self, reassembler: &mut StreamReassembler) {
        println!("  step: {}", ReassemblerExpectation::to_string(self));
        assert!(reassembler.stream_out().eof())
    }
}
impl ReassemblerExpectation for AtEof {
    fn description(&self) -> String {
        format!("at EOF")
    }
}

pub struct NotAtEof;
impl ReassemblerTestStep for NotAtEof {
    fn execute(&self, reassembler: &mut StreamReassembler) {
        println!("  step: {}", ReassemblerExpectation::to_string(self));
        assert!(!reassembler.stream_out().eof())
    }
}
impl ReassemblerExpectation for NotAtEof {
    fn description(&self) -> String {
        format!("not at EOF")
    }
}

pub struct SubmitSegment {
    data: String,
    index: SizeT,
    eof: bool,
}
impl ReassemblerTestStep for SubmitSegment {
    fn execute(&self, reassembler: &mut StreamReassembler) {
        println!("  step: {}", ReassemblerAction::to_string(self));
        reassembler.push_substring(&self.data, self.index as u64, self.eof)
    }
}
impl ReassemblerAction for SubmitSegment {
    fn description(&self) -> String {
        format!(
            "substring submitted with data \"{}\", index `{}`, eof `{}`",
            self.data, self.index, self.eof
        )
    }
}
impl SubmitSegment {
    #[allow(dead_code)]
    pub fn new(_data: String, _index: SizeT, _eof: bool) -> SubmitSegment {
        SubmitSegment {
            data: _data,
            index: _index,
            eof: _eof,
        }
    }

    #[allow(dead_code)]
    pub fn with_eof(&mut self, _eof: bool) -> &SubmitSegment {
        self.eof = _eof;
        self
    }
}

pub struct ReassemblerTestHarness {
    reassembler: StreamReassembler,
}
impl ReassemblerTestHarness {
    #[allow(dead_code)]
    pub fn new(capacity: SizeT) -> ReassemblerTestHarness {
        let harness = ReassemblerTestHarness {
            reassembler: StreamReassembler::new(capacity),
        };
        println!("test:=> Initialized (capacity = {})", capacity);
        harness
    }

    #[allow(dead_code)]
    pub fn execute(&mut self, step: &dyn ReassemblerTestStep) {
        step.execute(&mut self.reassembler)
    }
}
