use std::error::Error;
use tokio::runtime::Handle;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let _ = tokio::join!(
        tokio::spawn(busy::work()),
        tokio::spawn(busy::work()),
        taskdump(),
    );

    Ok(())

}

async fn taskdump() {
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    let _ = Handle::current().in_pause(|| {
        println!("PAUSE START");
        std::thread::sleep(std::time::Duration::from_millis(2000));
        println!("PAUSE END");
    });
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    std::process::exit(0);
}

mod busy {
    use futures::future::{BoxFuture, FutureExt};


    #[inline(never)]
    pub async fn work() {
        loop {
            println!("{:?}", std::thread::current().id());

            for i in 0..u8::MAX {
                recurse(i).await;
            }
        }
    }

    #[inline(never)]
    pub fn recurse(depth: u8) -> BoxFuture<'static, ()> {
        async move {
            tokio::task::yield_now().await;
            if depth == 0 {
                return;
            } else {
                recurse(depth - 1).await;
            }
        }.boxed()
    }
}