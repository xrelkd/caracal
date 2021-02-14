use std::sync::Mutex;

use snafu::ResultExt;
use x11rb::protocol::{xproto, xproto::ConnectionExt, Event};

use crate::{error, x11::Context, ClipboardLoad, Error};

#[derive(Debug)]
pub struct Getter {
    context: Mutex<Context>,
    property_atom: xproto::Atom,
}

impl Getter {
    pub fn new(context: Context) -> Result<Getter, Error> {
        let property_atom = context.get_intern_atom("CARACAL_CLIPBOARD")?;
        let context = Mutex::new(context);
        Ok(Getter { context, property_atom })
    }
}

impl ClipboardLoad for Getter {
    fn load(&self, mime: &mime::Mime) -> Result<Vec<u8>, Error> {
        let context = self.context.lock().unwrap();
        let data = context.load(self.property_atom, mime)?;
        Ok(data)
    }
}

impl Context {
    fn load(&self, property_atom: xproto::Atom, mime: &mime::Mime) -> Result<Vec<u8>, Error> {
        let mut is_incr = false;
        let mut buffer = Vec::new();

        let target_atom = self.get_intern_atom_from_mime(mime)?;
        let clipboard_type = self.clipboard_type();

        let selection_owner = self
            .connection
            .get_selection_owner(clipboard_type)
            .context(error::GetSelectionOwner)?
            .reply()
            .map(|r| r.owner)
            .context(error::Reply)?;
        if selection_owner == x11rb::NONE {
            return Err(Error::Empty);
        }

        self.connection
            .convert_selection(
                self.window,
                self.clipboard_type(),
                target_atom,
                property_atom,
                x11rb::CURRENT_TIME,
            )
            .context(error::ConvertSelection)?;
        self.flush_connection()?;

        while let Ok(event) = self.wait_for_event() {
            match event {
                Event::SelectionNotify(event) => {
                    if event.selection != clipboard_type {
                        continue;
                    }

                    if event.property == x11rb::NONE {
                        return Err(Error::MatchMime { expected_mime: mime.clone() });
                    }

                    let reply = self
                        .connection
                        .get_property(
                            false,
                            self.window,
                            event.property,
                            xproto::GetPropertyType::ANY,
                            buffer.len() as u32,
                            std::u32::MAX,
                        )
                        .context(error::GetProperty)?
                        .reply()
                        .context(error::GetPropertyReply)?;

                    if reply.type_ == self.atom_cache.incr {
                        if let Some(&size) = reply.value.get(0) {
                            buffer.reserve(size as usize);
                        }

                        self.connection
                            .delete_property(self.window, property_atom)
                            .context(error::DeleteProperty)?;
                        self.flush_connection()?;
                        is_incr = true;
                        continue;
                    } else if reply.type_ != target_atom {
                        return Err(Error::MatchMime { expected_mime: mime.clone() });
                    }

                    buffer.extend_from_slice(&reply.value);
                    break;
                }
                Event::PropertyNotify(event) if is_incr => {
                    if event.state != xproto::Property::NEW_VALUE {
                        continue;
                    }

                    let length = self
                        .connection
                        .get_property(
                            false,
                            self.window,
                            property_atom,
                            xproto::GetPropertyType::ANY,
                            0,
                            0,
                        )
                        .context(error::GetProperty)?
                        .reply()
                        .map(|r| r.bytes_after)
                        .context(error::Reply)?;

                    let reply = self
                        .connection
                        .get_property(
                            true,
                            self.window,
                            property_atom,
                            xproto::GetPropertyType::ANY,
                            0,
                            length,
                        )
                        .context(error::GetProperty)?
                        .reply()
                        .context(error::Reply)?;

                    if reply.type_ != target_atom {
                        continue;
                    }

                    buffer.extend_from_slice(&reply.value);
                    if reply.value_len == 0 {
                        break;
                    }
                }
                _ => continue,
            }
        }

        self.connection
            .delete_property(self.window, property_atom)
            .context(error::DeleteProperty)?;
        self.flush_connection()?;

        if buffer.is_empty() {
            return Err(Error::Empty);
        }

        Ok(buffer)
    }
}
