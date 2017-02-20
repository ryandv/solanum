use daemon::io::mio;

use daemon::CommandProcessor;
use daemon::PomodoroQueryMapper;
use daemon::io::CommandEventSubscriber;
use daemon::io::SignalEventSubscriber;
use daemon::io::EventPoller;
use daemon::pomodoros::Pomodoros;

use std::boxed::Box;
use std::io;
use std::ops::Deref;
use std::os::unix::io::RawFd;

pub struct DaemonContainer<'a> {
    event_poller: EventPoller<'a>,
    command_event_subscriber: CommandEventSubscriber<PomodoroQueryMapper>,
    signal_event_subscriber: SignalEventSubscriber<'a>,
}

impl<'a> DaemonContainer<'a> {
    pub fn new(signalfd: &'a RawFd) -> io::Result<DaemonContainer<'a>> {
        let query_mapper = PomodoroQueryMapper::new();
        let command_processor = CommandProcessor::new(query_mapper);
        let command_event_subscriber: CommandEventSubscriber<PomodoroQueryMapper> =
            try!(CommandEventSubscriber::new(command_processor, mio::Token(0)));

        let evented_signalfd = mio::unix::EventedFd(&signalfd);
        let signalfd_subscriber = SignalEventSubscriber::new(evented_signalfd, mio::Token(1));

        let event_poller = try!(EventPoller::new());

        Ok(DaemonContainer {
            event_poller: event_poller,
            command_event_subscriber: command_event_subscriber,
            signal_event_subscriber: signalfd_subscriber,
        })
    }

    pub fn start(&'a mut self) -> io::Result<()> {
        try!(self.event_poller.listen_for(&self.signal_event_subscriber));
        try!(self.event_poller.listen_for(&self.command_event_subscriber));

        self.event_poller.start_polling()
    }
}
