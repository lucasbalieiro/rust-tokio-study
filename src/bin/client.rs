use Command::*;
use bytes::Bytes;
use mini_redis::client;
use tokio::sync::{mpsc, oneshot};

type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        value: Bytes,
        resp: Responder<()>,
    },
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);
    let tx2 = tx.clone();

    let manager = tokio::spawn(async move {
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();

        while let Some(cmd) = rx.recv().await {
            match cmd {
                Get { key, resp } => {
                    let res =client.get(&key).await;

                    let _ = resp.send(res);
                }
                Set { key, value, resp } => {
                    let res = client.set(&key, value).await;

                    let _ = resp.send(res);
                }
            }
        }
    });

    let get_task = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Get {
            key: "foo".to_string(),
            resp: resp_tx
        };

        tx.send(cmd).await.unwrap();

        let res = resp_rx.await;
        println!("GOT = {:?}", res);
    });

    let set_task = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Set {
            key: "foo".to_string(),
            value: "bar".into(),
            resp: resp_tx
        };

        tx2.send(cmd).await.unwrap();

        let res = resp_rx.await;
        print!("GOT = {:?}", res);
    });

    get_task.await.unwrap();
    set_task.await.unwrap();
    manager.await.unwrap();
}
