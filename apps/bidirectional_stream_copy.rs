use libc::SHUT_WR;
use rust_sponge::byte_stream::ByteStream;
use rust_sponge::tcp_helpers::tcp_sponge_socket::AsLocalStreamSocketMut;
use rust_sponge::util::aeventloop::AEventLoop;
use rust_sponge::util::eventloop::{Direction, EventLoop};
use rust_sponge::util::file_descriptor::{AsFileDescriptor, AsFileDescriptorMut, FileDescriptor};
use rust_sponge::util::socket::AsSocketMut;
use rust_sponge::SizeT;
use std::cell::RefCell;
use std::cmp::min;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[allow(dead_code)]
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

    // note: values in a scope are dropped in the opposite order they are defined
    // ref: https://web.mit.edu/rust-lang_v1.25/arch/amd64_ubuntu1404/share/doc/rust/html/book/first-edition/lifetimes.html#lifetimes-1
    // if eventloop defined before socket_rc..., then compile error be like:
    //      | `inbound` dropped here while still borrowed
    //      | borrow might be used here, when `eventloop` is dropped and runs the destructor for type `EventLoop<'_>`
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

#[allow(dead_code)]
pub fn bidirectional_stream_copy_sponge(socket: &mut dyn AsLocalStreamSocketMut) {
    let max_copy_length: SizeT = 65536;
    let buffer_size: SizeT = 1048576;

    let socket_rc = socket.as_socket_mut();
    let socket_file_rc = Arc::new(Mutex::new(
        socket
            .as_socket_mut()
            .lock()
            .unwrap()
            .as_file_descriptor()
            .clone(),
    ));
    let input = Arc::new(Mutex::new(FileDescriptor::new(libc::STDIN_FILENO)));
    let output = Arc::new(Mutex::new(FileDescriptor::new(libc::STDOUT_FILENO)));
    let outbound = Arc::new(Mutex::new(ByteStream::new(buffer_size)));
    let inbound = Arc::new(Mutex::new(ByteStream::new(buffer_size)));
    let outbound_shutdown = Arc::new(AtomicBool::new(false));
    let inbound_shutdown = Arc::new(AtomicBool::new(false));

    let mut eventloop = AEventLoop::new();

    socket_rc
        .clone()
        .lock()
        .unwrap()
        .as_socket_mut()
        .set_blocking(false);
    input.clone().lock().unwrap().set_blocking(false);
    output.clone().lock().unwrap().set_blocking(false);

    // rule 1: read from stdin into outbound byte stream
    let outbound_ = outbound.clone();
    let input_ = input.clone();
    let outbound_1 = outbound.clone();
    let inbound_ = inbound.clone();
    let outbound_2 = outbound.clone();
    eventloop.add_rule(
        input.clone(),
        Direction::In,
        Box::new(move || {
            let mut outbound_guard = outbound_.lock().unwrap();
            let mut input_guard = input_.lock().unwrap();
            let length = outbound_guard.remaining_capacity() as u32;
            outbound_guard.write(&input_guard.read(length));
            if input_guard.eof() {
                outbound_guard.end_input();
            };
        }),
        Box::new(move || {
            let outbound_guard = outbound_1.lock().unwrap();
            let inbound_guard = inbound_.lock().unwrap();
            return !outbound_guard.error()
                && outbound_guard.remaining_capacity() > 0
                && !inbound_guard.error();
        }),
        Box::new(move || {
            let mut outbound_guard = outbound_2.lock().unwrap();
            outbound_guard.end_input();
        }),
    );

    // rule 2: read from outbound byte stream into socket
    let outbound_ = outbound.clone();
    let outbound_1 = outbound.clone();
    let outbound_2 = outbound.clone();
    let outbound_shutdown_ = outbound_shutdown.clone();
    let outbound_shutdown_1 = outbound_shutdown.clone();
    let socket_rc_ = socket_rc.clone();
    eventloop.add_rule(
        socket_file_rc.clone(),
        Direction::Out,
        Box::new(move || {
            let mut outbound_guard = outbound_.lock().unwrap();
            let mut socket_guard = socket_rc_.lock().unwrap();

            let bytes_to_write = min(max_copy_length, outbound_guard.buffer_size());
            let bytes_written =
                socket_guard.write(&outbound_guard.peek_output(bytes_to_write), false);
            outbound_guard.pop_output(bytes_written);

            if outbound_guard.eof() {
                socket_guard.shutdown(SHUT_WR);
                outbound_shutdown_.store(true, Ordering::SeqCst);
            };
        }),
        Box::new(move || {
            let outbound_guard = outbound_1.lock().unwrap();
            return !outbound_guard.buffer_empty()
                || (outbound_guard.eof() && !outbound_shutdown_1.load(Ordering::SeqCst));
        }),
        Box::new(move || {
            let mut outbound_guard = outbound_2.lock().unwrap();
            outbound_guard.end_input();
        }),
    );

    // rule 3: read from socket into inbound byte stream
    let outbound_ = outbound.clone();
    let inbound_ = inbound.clone();
    let inbound_1 = inbound.clone();
    let inbound_2 = inbound.clone();
    let socket_rc_ = socket_rc.clone();
    eventloop.add_rule(
        socket_file_rc.clone(),
        Direction::In,
        Box::new(move || {
            let mut inbound_guard = inbound_.lock().unwrap();
            let mut socket_guard = socket_rc_.lock().unwrap();

            let length = inbound_guard.remaining_capacity() as u32;
            inbound_guard.write(&socket_guard.read(length));
            if socket_guard.eof() {
                inbound_guard.end_input();
            };
        }),
        Box::new(move || {
            let inbound_guard = inbound_1.lock().unwrap();
            let outbound_guard = outbound_.lock().unwrap();
            return !inbound_guard.error()
                && inbound_guard.remaining_capacity() > 0
                && !outbound_guard.error();
        }),
        Box::new(move || {
            let mut inbound_guard = inbound_2.lock().unwrap();
            inbound_guard.end_input();
        }),
    );

    // rule 4: read from inbound byte stream into stdout
    let inbound_ = inbound.clone();
    let inbound_1 = inbound.clone();
    let inbound_2 = inbound.clone();
    let output_ = output.clone();
    let inbound_shutdown_ = inbound_shutdown.clone();
    let inbound_shutdown_1 = inbound_shutdown.clone();
    eventloop.add_rule(
        output.clone(),
        Direction::Out,
        Box::new(move || {
            let mut inbound_guard = inbound_.lock().unwrap();
            let mut output_guard = output_.lock().unwrap();

            let bytes_to_write = min(max_copy_length, inbound_guard.buffer_size());
            let bytes_written =
                output_guard.write(&inbound_guard.peek_output(bytes_to_write), false);
            inbound_guard.pop_output(bytes_written);

            if inbound_guard.eof() {
                output_guard.close();
                inbound_shutdown_.store(true, Ordering::SeqCst);
            };
        }),
        Box::new(move || {
            let inbound_gurad = inbound_1.lock().unwrap();
            return !inbound_gurad.buffer_empty()
                || (inbound_gurad.eof() && !inbound_shutdown_1.load(Ordering::SeqCst));
        }),
        Box::new(move || {
            let mut inbound_gurad = inbound_2.lock().unwrap();
            inbound_gurad.end_input();
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
