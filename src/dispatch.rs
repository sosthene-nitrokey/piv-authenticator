#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Transport {
    #[default]
    Apdu,
    #[cfg(feature = "ctaphid")]
    Ctaphid,
}

#[cfg(feature = "apdu-dispatch")]
mod adpu {
    use crate::{reply::Reply, Authenticator, /*constants::PIV_AID,*/ Result};

    use apdu_dispatch::{app::App, command, response, Command};
    use iso7816::{Interface, Status};

    use core::mem::replace;

    use super::Transport;

    impl<T: crate::Client> Authenticator<T> {
        fn apdu_call(&mut self, reply: Reply<'_, { command::SIZE }>) -> Result {
            let old = replace(&mut self.state.volatile.active_transport, Transport::Apdu);
            if old != Transport::Apdu {
                self.deselect();
                self.select(reply)?;
            }
            Ok(())
        }
    }

    impl<T> App<{ command::SIZE }, { response::SIZE }> for Authenticator<T>
    where
        T: crate::Client,
    {
        fn select(
            &mut self,
            interface: Interface,
            _apdu: &Command,
            reply: &mut response::Data,
        ) -> Result {
            if interface != Interface::Contact {
                return Err(Status::ConditionsOfUseNotSatisfied);
            }
            self.apdu_call(Reply(reply))?;
            self.select(Reply(reply))
        }

        fn deselect(&mut self) {
            self.deselect()
        }

        fn call(
            &mut self,
            interface: Interface,
            apdu: &Command,
            reply: &mut response::Data,
        ) -> Result {
            if interface != Interface::Contact {
                return Err(Status::ConditionsOfUseNotSatisfied);
            }
            self.respond(&apdu.as_view(), &mut Reply(reply))
        }
    }
}

#[cfg(feature = "ctaphid")]
mod ctaphid {
    use crate::{reply::Reply, Authenticator /*constants::PIV_AID,*/};

    use ctaphid_dispatch::app::App;
    use ctaphid_dispatch::command::{Command, VendorCommand};
    use ctaphid_dispatch::types::{AppResult, Error, Message};
    use iso7816::command::CommandView;
    use iso7816::Status;

    use core::mem::replace;

    use super::Transport;

    impl<T: crate::Client> Authenticator<T> {
        fn ctaphid_call(&mut self, mut reply: Reply<'_, 7609>) -> AppResult {
            let old = replace(
                &mut self.state.volatile.active_transport,
                Transport::Ctaphid,
            );
            if old != Transport::Ctaphid {
                self.deselect();
                match self.select(reply.lend()) {
                    Err(status) => {
                        reply.extend_from_slice(&status.to_u16().to_be_bytes()).ok();
                    }
                    Ok(()) => {}
                }
            }
            Ok(())
        }
    }

    const PIV_COMMAND: VendorCommand = VendorCommand::H73;

    impl<T: crate::Client> App<'static> for Authenticator<T> {
        fn commands(&self) -> &'static [Command] {
            &[Command::Vendor(PIV_COMMAND)]
        }

        fn interrupt(&self) -> Option<&'static trussed::interrupt::InterruptFlag> {
            self.trussed.interrupt()
        }

        fn call(
            &mut self,
            command: Command,
            request: &Message,
            response: &mut Message,
        ) -> AppResult {
            if command != Command::Vendor(PIV_COMMAND) {
                return Err(Error::InvalidCommand);
            }

            self.ctaphid_call(Reply(response))?;

            let command = CommandView::try_from(&**request).map_err(|_err| {
                warn!("Failed to parse request: {_err:?}");
                Error::InvalidCommand
            })?;

            let status = match self.respond(&command, response) {
                Err(status) => status,
                Ok(()) => Status::Success,
            };

            response
                .extend_from_slice(&status.to_u16().to_be_bytes())
                .ok();

            Ok(())
        }
    }
}
