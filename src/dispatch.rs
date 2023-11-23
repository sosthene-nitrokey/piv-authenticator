#[cfg(feature = "apdu-dispatch")]
mod adpu {
    use crate::{reply::Reply, Authenticator, /*constants::PIV_AID,*/ Result};

    use apdu_dispatch::{app::App, command, response, Command};
    use iso7816::{Interface, Status};

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
            self.respond(apdu, &mut Reply(reply))
        }
    }
}

#[cfg(feature = "ctaphid")]
mod ctaphid {
    use crate::{reply::Reply, Authenticator, /*constants::PIV_AID,*/ Result};

    use ctaphid_dispatch::app::App;
    use ctaphid_dispatch::command::{Command, VendorCommand};
    use ctaphid_dispatch::types::{AppResult, Error, Message};
    use iso7816::command::CommandView;
    use iso7816::Status;

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
