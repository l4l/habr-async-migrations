use std::fmt::Display;

use new_futures::compat::Compat01As03 as Compat;
use new_futures::StreamExt as _;
use old_futures::Stream as OldStream;

async fn stream_iterate<E>(
    old_stream: impl OldStream<Item = impl Display, Error = E>,
) -> Result<(), E> {
    let stream = Compat::new(old_stream);
    let mut stream = Box::pin(stream);

    while let Some(item) = stream.as_mut().next().await.transpose()? {
        println!("{}", item);
    }

    Ok(())
}

use new_futures::task::Spawn;
use new_futures::FutureExt as _;

fn main() {
    let mut pool = new_futures::executor::LocalPool::new();

    let old_stream = old_futures::stream::iter_ok::<_, ()>(1..15);

    let new_stream = Box::pin(stream_iterate(old_stream).map(|_| ()));
    let new_fut = new_futures::task::FutureObj::new(new_stream);

    // Для одной таски можно также воспользоваться `LocalPool::run_util`
    // без дополнительных оберток и аллокаций
    pool.spawner().spawn_obj(new_fut).unwrap();

    pool.run();
}
