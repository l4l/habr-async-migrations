use std::time::Duration;

use futures::channel::mpsc;
use futures::prelude::*;
use tokio::timer;

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel(32);

    tokio::spawn(sink_fibb_series(tx));

    rx.take(100)
        .for_each(|val| {
            println!("{}", val);
            async {
                timer::delay_for(Duration::from_millis(50)).await;
            }
        })
        .await;
}

pub async fn sink_fibb_series(sink: impl Sink<u32>) {
    stream::unfold((1u32, 1), |(mut fact, n)| {
        async move {
            while fact.checked_mul(n).is_none() {
                fact >>= 1;
            }
            fact *= n;
            Some((fact, (fact, n + 1)))
        }
    })
    .map(Ok)
    .forward(sink)
    .map(|_v| ())
    .await
}

pub async fn sink_fibb_series_typed(sink: impl Sink<u32>) {
    FactStream::new().map(Ok).forward(sink).map(|_v| ()).await
}

use std::marker::PhantomPinned;

struct Fact {
    inner: u32,
    _pin: PhantomPinned,
}

impl From<u32> for Fact {
    fn from(inner: u32) -> Self {
        Self {
            inner,
            _pin: PhantomPinned,
        }
    }
}

use pin_project::pin_project;

#[pin_project]
struct FactStream {
    fact: Fact,
    n: u32,
}

impl FactStream {
    fn new() -> Self {
        Self {
            fact: 1.into(),
            n: 1,
        }
    }
}

use std::pin::Pin;
use std::task::{Context, Poll};

impl Stream for FactStream {
    type Item = u32;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        while this.fact.inner.checked_mul(*this.n).is_none() {
            this.fact.inner >>= 1;
        }
        this.fact.inner *= *this.n;
        *this.n += 1;

        Poll::Ready(Some(this.fact.inner))
    }
}
