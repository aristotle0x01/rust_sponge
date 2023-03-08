use crate::byte_stream::ByteStream;
use crate::SizeT;
use std::cmp::{max, min};
use std::collections::{BTreeMap, LinkedList};

#[derive(Debug)]
pub struct StreamReassembler {
    next_stream_index: u64,
    ending_index: u64,
    reassemble_count: SizeT,
    ended: bool,
    capacity: SizeT,
    buffer: Vec<u8>,
    marker_map: BTreeMap<u64, u64>,
    output: ByteStream,
}
impl StreamReassembler {
    #[allow(dead_code)]
    pub fn new(_capacity: SizeT) -> StreamReassembler {
        StreamReassembler {
            next_stream_index: 0,
            ending_index: 0,
            reassemble_count: 0,
            ended: false,
            capacity: _capacity,
            buffer: vec![0; _capacity],
            marker_map: BTreeMap::new(),
            output: ByteStream::new(_capacity),
        }
    }

    #[allow(dead_code)]
    pub fn push_substring(&mut self, data: &[u8], index: u64, eof: bool) {
        if eof {
            self.ending_index = index + data.len() as u64;
            self.ended = true;
        }

        // always put into reassemble buffer first, then reassemble to in-order bytes
        // data exceed capacity will be discarded, reserve earlier bytes first
        while !data.is_empty() && self.output.remaining_capacity() > 0 {
            // valid: [index|_next_stream_index, _next_stream_index + _output.remaining_capacity() | index + data.length()]
            let data_start_stream_index = if self.next_stream_index > index {
                self.next_stream_index
            } else {
                index
            };
            let data_end_stream_index: u64 = min(
                self.next_stream_index + self.output.remaining_capacity() as u64,
                index + data.len() as u64,
            );
            if data_end_stream_index <= data_start_stream_index {
                break;
            }

            let it = self.marker_map.get(&data_start_stream_index);
            if it.is_some() {
                if data_end_stream_index > *it.unwrap() {
                    self.marker_map
                        .insert(data_start_stream_index, data_end_stream_index);
                }
            } else {
                self.marker_map
                    .insert(data_start_stream_index, data_end_stream_index);
            }

            let count: SizeT = (data_end_stream_index - data_start_stream_index) as SizeT;
            let d_index: SizeT = (data_start_stream_index - index) as SizeT;
            let begin_index: SizeT = (data_start_stream_index % (self.capacity as u64)) as SizeT;
            if count <= (self.capacity - begin_index) {
                let writable = &data[d_index..(d_index + count)];
                self.buffer[begin_index..(begin_index + count)].copy_from_slice(writable);
            } else {
                let size_1: SizeT = self.capacity - begin_index;
                let writable1 = &data[d_index..(d_index + size_1)];
                self.buffer[begin_index..self.capacity].copy_from_slice(writable1);

                let size_2: SizeT = count - size_1;
                let writable2 = &data[(d_index + size_1)..(d_index + size_1 + size_2)];
                self.buffer[0..size_2].copy_from_slice(writable2);
            }

            self.recount();

            break;
        }

        self.reassemble();

        // only when _next_stream_index surpassing _ending_index will stream be ended
        if self.ended && self.next_stream_index >= self.ending_index {
            self.output.end_input();
        }
    }

    #[allow(dead_code)]
    pub fn stream_out_mut(&mut self) -> &mut ByteStream {
        &mut self.output
    }

    #[allow(dead_code)]
    pub fn stream_out(&self) -> &ByteStream {
        &self.output
    }

    #[allow(dead_code)]
    pub fn unassembled_bytes(&self) -> SizeT {
        self.reassemble_count
    }

    #[allow(dead_code)]
    pub fn empty(&self) -> bool {
        self.reassemble_count == 0
    }

    #[allow(dead_code)]
    fn reassemble(&mut self) {
        // merge the fist consecutive range
        let mut min_index: u64 = 0;
        let mut max_index: u64 = 0;
        let mut d_list: LinkedList<u64> = LinkedList::new();
        for (first, second) in &self.marker_map {
            if max_index == 0 {
                if *first > self.next_stream_index {
                    return;
                }
                min_index = *first;
                max_index = *second;
                d_list.push_back(*first);
            } else if *first > max_index {
                break;
            } else if *first == max_index {
                max_index = *second;
                d_list.push_back(*first);
            } else if *second > max_index {
                max_index = *second;
                d_list.push_back(*first);
            } else {
                d_list.push_back(*first);
            }
        }
        if max_index > 0 {
            for n in d_list {
                self.marker_map.remove(&n);
            }
            self.marker_map.insert(min_index, max_index);
        }

        let remaining_capacity = self.output.remaining_capacity();

        let kv = self.marker_map.iter().next();
        if kv.is_none() {
            return;
        }

        let (k, v) = kv.unwrap();
        let (first, second) = (*k, *v);
        if !(first <= self.next_stream_index && second > self.next_stream_index) {
            return;
        }
        let count: SizeT = min(
            remaining_capacity,
            (second - self.next_stream_index) as SizeT,
        );
        if count == 0 {
            return;
        }

        let mut r = vec![0u8; count];
        let begin_index: SizeT = (self.next_stream_index % self.capacity as u64) as SizeT;
        if count <= (self.capacity - begin_index) {
            r.copy_from_slice(&self.buffer[begin_index..(begin_index + count)]);
        } else {
            let size_1 = self.capacity - begin_index;
            r[0..size_1].copy_from_slice(&self.buffer[begin_index..(begin_index + size_1)]);

            let size_2 = count - size_1;
            r[size_1..count].copy_from_slice(&self.buffer[0..size_2]);
        }
        self.output.write(r.as_slice());

        if (self.next_stream_index + count as u64) == second {
            self.marker_map.remove(&first);
        } else {
            self.marker_map.clear();
        }
        self.next_stream_index = self.next_stream_index + count as u64;

        self.recount();
    }

    #[allow(dead_code)]
    fn recount(&mut self) {
        self.reassemble_count = 0;

        let mut last_right: u64 = 0;
        let valid_last: u64 = self.next_stream_index + self.output.remaining_capacity() as u64;

        for (first, second) in &self.marker_map {
            if *first >= last_right || *second > last_right {
                let tmp: u64 = max(*first, self.next_stream_index);
                let last: u64 = min(*second, valid_last);

                self.reassemble_count += (last - tmp) as SizeT;
                last_right = last;
            }
        }
    }
}
