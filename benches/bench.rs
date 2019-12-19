#![feature(test, async_closure)]

extern crate test;
extern crate twinkle;
use futures::future::join;
use test::Bencher;


#[bench]
fn twinkle_set_bench(b: &mut Bencher) {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let (mut client, mut listener) = rt.block_on(async {twinkle::open("127.0.0.1:3000".to_string()).await.unwrap()});

    rt.spawn(async move{listener.listen();});
    b.bench(|_|{
        rt.block_on(async {
            for _ in 0..10 {
                client.ping().await;
            };
        });
    });
}

