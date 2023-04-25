// Copyright (C) 2022  Nitrokey GmbH
// SPDX-License-Identifier: LGPL-3.0-only

use piv_authenticator::{virt::with_ram_client, vpicc::VpiccCard, Authenticator, Options};

use std::{sync::mpsc, thread::sleep, time::Duration};
use stoppable_thread::spawn;

use std::sync::Mutex;

static VSC_MUTEX: Mutex<()> = Mutex::new(());

pub fn with_vsc<F: FnOnce() -> R, R>(f: F) -> R {
    let _lock = VSC_MUTEX.lock().unwrap();

    let mut vpicc = vpicc::connect().expect("failed to connect to vpcd");

    let (tx, rx) = mpsc::channel();
    let handle = spawn(move |stopped| {
        with_ram_client("opcard", |client| {
            let card = Authenticator::new(client, Options::default());
            let mut vpicc_card = VpiccCard::new(card);
            let mut result = Ok(());
            while !stopped.get() && result.is_ok() {
                result = vpicc.poll(&mut vpicc_card);
                if result.is_ok() {
                    tx.send(()).expect("failed to send message");
                }
            }
            result
        })
    });

    rx.recv().expect("failed to read message");

    sleep(Duration::from_millis(200));

    let result = f();

    handle
        .stop()
        .join()
        .expect("failed to join vpicc thread")
        .expect("failed to run vpicc smartcard");
    result
}