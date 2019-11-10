use std::time::{Duration, Instant};

use futures::prelude::*;
use futures::sync::mpsc;
use tokio::runtime::Runtime;
use tokio::timer::Delay;

fn main() {
    let mut rt = Runtime::new().unwrap();

    let (tx, rx) = mpsc::channel(32);
    rt.spawn(Box::new(sink_fibb_series(tx.sink_map_err(|_e| ()))));
    let fut = rx.take(100).for_each(|val| {
        println!("{}", val);
        Delay::new(Instant::now() + Duration::from_millis(50))
            .map(|_| ())
            .map_err(|_| ())
    });
    rt.spawn(Box::new(fut));

    rt.shutdown_on_idle().wait().unwrap();
}

use futures::{future, stream};

pub fn sink_fibb_series(
    sink: impl Sink<SinkItem = u32, SinkError = ()>,
) -> impl Future<Item = (), Error = ()> {
    stream::unfold((1u32, 1), |(mut fact, n)| {
        while fact.checked_mul(n).is_none() {
            fact >>= 1;
        }
        fact *= n;
        Some(future::ok((fact, (fact, n + 1))))
    })
    .forward(sink)
    .map(|_v| ())
}

pub fn sink_fibb_series_with_type(
    sink: impl Sink<SinkItem = u32, SinkError = ()>,
) -> impl Future<Item = (), Error = ()> {
    FactStream::new().forward(sink).map(|_v| ())
}

struct FactStream {
    fact: u32,
    n: u32,
}

impl FactStream {
    fn new() -> Self {
        Self { fact: 1, n: 1 }
    }
}

impl Stream for FactStream {
    type Item = u32;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        while self.fact.checked_mul(self.n).is_none() {
            self.fact >>= 1;
        }
        self.fact *= self.n;
        self.n += 1;
        Ok(Async::Ready(Some(self.fact)))
    }
}
