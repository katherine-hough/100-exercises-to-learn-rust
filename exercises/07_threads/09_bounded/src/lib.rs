// TODO: Convert the implementation to use bounded channels.
use crate::data::{Ticket, TicketDraft};
use crate::store::{TicketId, TicketStore};
use std::sync::mpsc::{sync_channel, TrySendError};
use std::sync::mpsc::{Receiver, SyncSender};

pub mod data;
pub mod store;

#[derive(Debug, thiserror::Error)]
pub enum TicketStoreError {
    #[error("The ticket store is overloaded")]
    ChannelFullError,
    #[error("The ticket store was closed")]
    ConnectionError,
}

impl From<TrySendError<Command>> for TicketStoreError {
    fn from(value: TrySendError<Command>) -> Self {
        match value {
            TrySendError::Full(_) => Self::ChannelFullError,
            TrySendError::Disconnected(_) => Self::ConnectionError,
        }
    }
}

#[derive(Clone)]
pub struct TicketStoreClient {
    sender: SyncSender<Command>,
}

impl TicketStoreClient {
    pub fn insert(&self, draft: TicketDraft) -> Result<TicketId, TicketStoreError> {
        let (r_sender, r_receiver) = sync_channel(1);
        let command = Command::Insert {
            draft,
            response_channel: r_sender,
        };
        self.sender
            .try_send(command)
            .map_err(TicketStoreError::from)?;
        Ok(r_receiver.recv().unwrap())
    }

    pub fn get(&self, id: TicketId) -> Result<Option<Ticket>, TicketStoreError> {
        let (r_sender, r_receiver) = sync_channel(1);
        let command = Command::Get {
            id,
            response_channel: r_sender,
        };
        self.sender
            .try_send(command)
            .map_err(TicketStoreError::from)?;
        Ok(r_receiver.recv().unwrap())
    }
}

pub fn launch(capacity: usize) -> TicketStoreClient {
    let (sender, receiver) = sync_channel(capacity);
    std::thread::spawn(move || server(receiver));
    TicketStoreClient { sender }
}

enum Command {
    Insert {
        draft: TicketDraft,
        response_channel: SyncSender<TicketId>,
    },
    Get {
        id: TicketId,
        response_channel: SyncSender<Option<Ticket>>,
    },
}

pub fn server(receiver: Receiver<Command>) {
    let mut store = TicketStore::new();
    loop {
        match receiver.recv() {
            Ok(Command::Insert {
                draft,
                response_channel,
            }) => {
                let id = store.add_ticket(draft);
                response_channel.send(id);
            }
            Ok(Command::Get {
                id,
                response_channel,
            }) => {
                let ticket = store.get(id);
                response_channel.send(ticket.cloned());
            }
            Err(_) => {
                // There are no more senders, so we can safely break
                // and shut down the server.
                break;
            }
        }
    }
}
