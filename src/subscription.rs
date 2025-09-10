use crate::{backend::BackendCommand, Command, Event};
use alacritty_terminal::event::Event as AlacrittyEvent;
use iced::futures::{SinkExt, Stream};
use iced_graphics::futures::subscription;
use std::hash::Hash;
use crossbeam_channel::Receiver;

pub struct Subscription {
    term_id: u64,
    backend_event_rx: Receiver<AlacrittyEvent>,
}

impl Subscription {
    pub fn new(term_id: u64, backend_event_rx: Receiver<AlacrittyEvent>) -> Self {
        Self { term_id, backend_event_rx }
    }

    pub fn event_stream(&self) -> impl Stream<Item = Event> {
        let term_id = self.term_id;
        let backend_event_rx = self.backend_event_rx.clone();
        iced::stream::channel(100, move |mut output| async move {
            // let (event_tx, mut event_rx) = mpsc::channel(100);
            // let cmd = Command::InitBackend(event_tx);
            // output
            //     .send(Event::CommandReceived(term_id, cmd))
            //     .await
            //     .unwrap_or_else(|_| {
            //         panic!("iced_term stream {}: sending BackendEventSenderReceived event is failed", term_id)
            //     });

            let mut shutdown = false;
            loop {
                match backend_event_rx.try_recv() {
                    Ok(event) => {
                        if let AlacrittyEvent::Exit = event {
                            shutdown = true
                        };

                        let cmd = Command::ProcessBackendCommand(
                            BackendCommand::ProcessAlacrittyEvent(event),
                        );

                        println!("subscription: {:?}", cmd);

                        output
                            .send(Event::CommandReceived(term_id, cmd))
                            .await
                            .unwrap_or_else(|_| {
                                panic!("iced_term stream {}: sending BackendEventReceived event is failed", term_id)
                            });
                    },
                    Err(_) => {
                        if !shutdown {
                            panic!("iced_term stream {}: terminal event channel closed unexpected", term_id);
                        }
                    },
                }
            }
        })
    }
}

impl subscription::Recipe for Subscription {
    type Output = Event;

    fn hash(&self, state: &mut subscription::Hasher) {
        self.term_id.hash(state);
    }

    fn stream(
        self: Box<Self>,
        _: subscription::EventStream,
    ) -> iced_graphics::futures::BoxStream<Self::Output> {
        Box::pin(self.event_stream())
    }
}
