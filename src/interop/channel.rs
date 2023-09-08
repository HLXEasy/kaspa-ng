use workflow_core::channel::{Sender as ChannelSender,Receiver,unbounded,bounded,RecvError,TryRecvError,SendError,TrySendError};

#[derive(Debug, Clone)]
pub struct Sender<T> {
    ctx : egui::Context,
    sender: ChannelSender<T>,
}

impl<T> Sender<T> {

    pub fn new(ctx : egui::Context, sender: ChannelSender<T>) -> Self {
        Self { ctx, sender }
    }

    pub async fn send(&self, msg: T) -> Result<(), SendError<T>> {
        self.sender.send(msg).await?;
        self.ctx.request_repaint();
        Ok(())
    }

    pub fn try_send(&self, msg: T) -> Result<(), TrySendError<T>> {
        self.sender.try_send(msg)?;
        self.ctx.request_repaint();
        Ok(())
    }

    pub fn receiver_count(&self) -> usize {
        self.sender.receiver_count()
    }

    pub fn sender_count(&self) -> usize {
        self.sender.sender_count()
    }
}

// impl<T> From<ChannelSender<T>> for Sender<T> {
//     fn from(sender: ChannelSender<T>, ctx : egui::Context) -> Self {
//         Self { sender, ctx }
//     }
// }

#[derive(Debug, Clone)]
pub struct Channel<T = ()> {
    pub sender: Sender<T>,
    pub receiver: Receiver<T>,
}

impl<T> Channel<T> {
    pub fn unbounded(ctx: &egui::Context) -> Self {
        let (sender, receiver) = unbounded();
        Self { sender : Sender::new(ctx.clone(),sender), receiver }
    }

    // pub fn bounded(cap: usize) -> Self {
    //     let (sender, receiver) = bounded(cap);
    //     Self { sender : sender.into(), receiver }
    // }

    // pub fn oneshot() -> Self {
    //     let (sender, receiver) = bounded(1);
    //     Self { sender : sender.into(), receiver }
    // }

    pub fn drain(&self) -> std::result::Result<(), TryRecvError> {
        while !self.receiver.is_empty() {
            self.receiver.try_recv()?;
        }
        Ok(())
    }

    pub async fn recv(&self) -> Result<T, RecvError> {
        self.receiver.recv().await
    }

    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        self.receiver.try_recv()
    }

    pub async fn send(&self, msg: T) -> Result<(), SendError<T>> {
        self.sender.send(msg).await
    }

    pub fn try_send(&self, msg: T) -> Result<(), TrySendError<T>> {
        self.sender.try_send(msg)
    }

    pub fn len(&self) -> usize {
        self.receiver.len()
    }

    pub fn is_empty(&self) -> bool {
        self.receiver.is_empty()
    }

    pub fn receiver_count(&self) -> usize {
        self.sender.receiver_count()
    }

    pub fn sender_count(&self) -> usize {
        self.sender.sender_count()
    }

    pub fn iter(&self) -> ChannelIterator<T> {
        ChannelIterator::new(self.receiver.clone())
    }
}

pub struct ChannelIterator<T> {
    receiver: Receiver<T>,
}

impl<T> ChannelIterator<T> {
    pub fn new(receiver: Receiver<T>) -> Self {
        ChannelIterator { receiver }
    }
}

impl<T> Iterator for ChannelIterator<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if self.receiver.is_empty() {
            None
        } else {
            self.receiver.try_recv().ok()
        }
    }
}