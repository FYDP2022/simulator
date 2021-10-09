use std::sync::{Mutex, MutexGuard};

use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures::StreamExt;

use async_tungstenite::async_std::connect_async;
use tungstenite::Error;

pub struct Message;

pub struct Client {
  _send_queue: UnboundedSender<Message>,
  receive_queue: Mutex<UnboundedReceiver<Message>>,
}

impl Client {
  pub async fn new() -> Result<Self, Error> {
    let (send_tx, send_rx) = futures::channel::mpsc::unbounded();
    let (receive_tx, receive_rx) = futures::channel::mpsc::unbounded();
    let (ws_stream, _) = connect_async("ws://127.0.0.1:9001").await?;
    let (write, read) = ws_stream.split();

    async_std::task::spawn(async move {
      read.map(|_msg| Ok(Message {})).forward(receive_tx).await.unwrap();
    });

    async_std::task::spawn(async move {
      send_rx
        .map(|_msg| Ok(tungstenite::Message::Text("hello world".into())))
        .forward(write)
        .await
        .unwrap();
    });

    Ok(Self {
      _send_queue: send_tx,
      receive_queue: Mutex::new(receive_rx),
    })
  }

  pub fn _send(&self, message: Message) {
    self._send_queue.unbounded_send(message).unwrap();
  }

  pub fn stream(&self) -> MutexGuard<UnboundedReceiver<Message>> {
    self.receive_queue.lock().unwrap()
  }
}
