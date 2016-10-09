extern crate mio;

use daemon::EventPoller;
use daemon::CommandEventSubscriber;
use daemon::CommandProcessor;
use daemon::SignalEventSubscriber;

use std::io;
use std::os::unix::io::RawFd;

pub struct DaemonContainer<'a> {
    event_poller: EventPoller<'a>,
    command_event_subscriber: CommandEventSubscriber,
    signal_event_subscriber: SignalEventSubscriber<'a>
}

impl<'a> DaemonContainer<'a> {
    pub fn new(signalfd: &'a RawFd) -> io::Result<DaemonContainer<'a>> {
        let command_processor = CommandProcessor::new();
        let command_event_subscriber = try!(
            CommandEventSubscriber::new(
                command_processor,
                mio::Token(0)
            )
        );

        let evented_signalfd = mio::unix::EventedFd(&signalfd);
        let signalfd_subscriber = SignalEventSubscriber::new(
            evented_signalfd,
            mio::Token(1)
        );

        let event_poller = try!(EventPoller::new());

        Ok(DaemonContainer {
            event_poller: event_poller,
            command_event_subscriber: command_event_subscriber,
            signal_event_subscriber: signalfd_subscriber
        })
    }

    pub fn start(&'a mut self) -> io::Result<()> {
        try!(self.event_poller.listen_for(&self.signal_event_subscriber));
        try!(self.event_poller.listen_for(&self.command_event_subscriber));

        self.event_poller.start_polling()
    }
}

