// {{{ Copyright (c) Paul R. Tagliamonte <paultag@gmail.com>, 2023-2024
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE. }}}

// #![feature(io_error_more)]
// #![feature(map_try_insert)]

mod zero;
use zero::Zero;

use arigato::server::AsyncServer;
use std::str::FromStr;
use tracing_subscriber::{fmt::format::FmtSpan, FmtSubscriber};

#[tokio::main]
async fn main() {
    let log_level = "info";

    let subscriber = FmtSubscriber::builder()
        .with_writer(std::io::stderr)
        .with_max_level(tracing::Level::from_str(log_level).unwrap())
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let args: Vec<String> = std::env::args().collect();
    let srv = AsyncServer::builder()
        .with_tcp_listen_address(&args[1])
        .with_msize(24 + (512 * 1024))
        .with_filesystem("", Zero::new())
        .build()
        .await
        .unwrap();

    srv.serve().await.unwrap();
}

// vim: foldmethod=marker
