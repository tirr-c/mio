use crate::{expect_events, sleep_ms};
use mio::event::Event;
use mio::net::UdpSocket;
use mio::{Events, Interests, Poll, PollOpt, Ready, Token};

#[test]
pub fn test_udp_level_triggered() {
    let mut poll = Poll::new().unwrap();
    let poll = &mut poll;
    let mut events = Events::with_capacity(1024);
    let events = &mut events;

    // Create the listener
    let tx = UdpSocket::bind(&"127.0.0.1:0".parse().unwrap()).unwrap();
    let rx = UdpSocket::bind(&"127.0.0.1:0".parse().unwrap()).unwrap();

    poll.registry()
        .register(
            &tx,
            Token(0),
            Interests::readable() | Interests::writable(),
            PollOpt::level(),
        )
        .unwrap();

    poll.registry()
        .register(
            &rx,
            Token(1),
            Interests::readable() | Interests::writable(),
            PollOpt::level(),
        )
        .unwrap();

    for _ in 0..2 {
        expect_events(
            poll,
            events,
            2,
            vec![
                Event::new(Ready::writable(), Token(0)),
                Event::new(Ready::writable(), Token(1)),
            ],
        );
    }

    tx.send_to(b"hello world!", &rx.local_addr().unwrap())
        .unwrap();

    sleep_ms(250);

    for _ in 0..2 {
        expect_events(
            poll,
            events,
            2,
            vec![Event::new(Ready::readable() | Ready::writable(), Token(1))],
        );
    }

    let mut buf = [0; 200];
    while rx.recv_from(&mut buf).is_ok() {}

    for _ in 0..2 {
        expect_events(
            poll,
            events,
            4,
            vec![Event::new(Ready::writable(), Token(1))],
        );
    }

    tx.send_to(b"hello world!", &rx.local_addr().unwrap())
        .unwrap();
    sleep_ms(250);

    expect_events(
        poll,
        events,
        10,
        vec![Event::new(Ready::readable() | Ready::writable(), Token(1))],
    );

    drop(rx);
}
