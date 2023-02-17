use libc::SHUT_WR;
use rust_sponge::byte_stream::ByteStream;
use rust_sponge::util::eventloop::{Direction, EventLoop};
use rust_sponge::util::file_descriptor::FileDescriptor;
use rust_sponge::util::socket::AsSocketMut;
use rust_sponge::SizeT;
use std::cell::RefCell;
use std::cmp::min;
use std::rc::Rc;

pub fn bidirectional_stream_copy(socket: &mut dyn AsSocketMut) {
    let max_copy_length: SizeT = 65536;
    let buffer_size: SizeT = 1048576;

    // Rc<RefCell<xxx>> way works here because it is used single-threaded-ly in this project
    
    let socket_rc = Rc::new(RefCell::new(socket.as_file_descriptor().clone()));
    let input = Rc::new(RefCell::new(FileDescriptor::new(libc::STDIN_FILENO)));
    let output = Rc::new(RefCell::new(FileDescriptor::new(libc::STDOUT_FILENO)));
    let outbound = Rc::new(RefCell::new(ByteStream::new(buffer_size)));
    let inbound = Rc::new(RefCell::new(ByteStream::new(buffer_size)));
    let outbound_shutdown = Rc::new(RefCell::new(false));
    let inbound_shutdown = Rc::new(RefCell::new(false));

    let mut eventloop = EventLoop::new();

    socket.set_blocking(false);
    input.borrow_mut().set_blocking(false);
    output.borrow_mut().set_blocking(false);

    // rule 1: read from stdin into outbound byte stream
    eventloop.add_rule(
        input.clone(),
        Direction::In,
        Box::new(|| {
            let outbound_ = outbound.clone();
            let length = outbound_.borrow().remaining_capacity() as u32;
            let input_ = input.clone();
            outbound_
                .borrow_mut()
                .write(&input_.borrow_mut().read(length));
            if input_.borrow().eof() {
                outbound_.borrow_mut().end_input();
            };
        }),
        Box::new(|| {
            let outbound_ = outbound.clone();
            return !outbound_.borrow().error()
                && outbound_.borrow().remaining_capacity() > 0
                && !inbound.clone().borrow().error();
        }),
        Box::new(|| {
            outbound.clone().borrow_mut().end_input();
        }),
    );

    // rule 2: read from outbound byte stream into socket
    eventloop.add_rule(
        socket_rc.clone(),
        Direction::Out,
        Box::new(|| {
            let outbound_ = outbound.clone();

            let bytes_to_write = min(max_copy_length, outbound_.borrow().buffer_size());
            let bytes_written =
                socket.write(&outbound_.borrow().peek_output(bytes_to_write), false);
            outbound_.borrow_mut().pop_output(bytes_written);

            if outbound_.borrow().eof() {
                socket.shutdown(SHUT_WR);
                *outbound_shutdown.borrow_mut() = true;
            };
        }),
        Box::new(|| {
            let outbound_ = outbound.clone();
            return !outbound_.borrow().buffer_empty()
                || (outbound_.borrow().eof() && !*outbound_shutdown.borrow());
        }),
        Box::new(|| {
            outbound.clone().borrow_mut().end_input();
        }),
    );

    // rule 3: read from socket into inbound byte stream
    eventloop.add_rule(
        socket_rc.clone(),
        Direction::In,
        Box::new(|| {
            let inbound_ = inbound.clone();

            let length = inbound_.borrow().remaining_capacity() as u32;
            inbound_
                .borrow_mut()
                .write(&socket_rc.clone().borrow_mut().read(length));
            if socket_rc.clone().borrow().eof() {
                inbound_.borrow_mut().end_input();
            };
        }),
        Box::new(|| {
            let inbound_ = inbound.clone();
            return !inbound_.borrow().error()
                && inbound_.borrow().remaining_capacity() > 0
                && !outbound.clone().borrow().error();
        }),
        Box::new(|| {
            inbound.clone().borrow_mut().end_input();
        }),
    );

    // rule 4: read from inbound byte stream into stdout
    eventloop.add_rule(
        output.clone(),
        Direction::Out,
        Box::new(|| {
            let inbound_ = inbound.clone();
            let output_ = output.clone();

            let bytes_to_write = min(max_copy_length, inbound_.borrow().buffer_size());
            let bytes_written = output_
                .borrow_mut()
                .write(&inbound_.borrow().peek_output(bytes_to_write), false);
            inbound_.borrow_mut().pop_output(bytes_written);

            if inbound_.borrow().eof() {
                output_.borrow_mut().close();
                *inbound_shutdown.borrow_mut() = true;
            };
        }),
        Box::new(|| {
            let inbound_ = inbound.clone();
            return !inbound_.borrow().buffer_empty()
                || (inbound_.borrow().eof() && !*inbound_shutdown.borrow());
        }),
        Box::new(|| {
            inbound.clone().borrow_mut().end_input();
        }),
    );

    // loop until completion
    loop {
        if rust_sponge::util::eventloop::Result::Exit == eventloop.wait_next_event(-1) {
            return;
        }
    }
}

fn main() {}
