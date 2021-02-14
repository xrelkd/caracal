#![cfg(feature = "x11")]

use std::sync::Arc;

use snafu::ResultExt;
use x11rb::{
    connection::Connection,
    protocol::{xproto, xproto::ConnectionExt},
    rust_connection::RustConnection,
};

use crate::{
    error, ClipboardLoad, ClipboardStore, ClipboardSubscribe, Error, MimeData, Mode, Subscriber,
};

mod getter;
mod setter;
mod watcher;

use self::{getter::Getter, setter::Setter, watcher::Watcher};

#[derive(Clone, Debug)]
pub struct Clipboard {
    getter: Arc<Getter>,
    setter: Arc<Setter>,
    watcher: Arc<Watcher>,
}

impl Clipboard {
    pub fn new(display_name: Option<&str>, mode: Mode) -> Result<Clipboard, Error> {
        let context = Context::new(display_name, mode)?;
        let getter = Arc::new(Getter::new(context.try_clone()?)?);
        let setter = Arc::new(Setter::new(context.try_clone()?)?);
        let watcher = Arc::new(Watcher::new(context)?);
        Ok(Clipboard { getter, setter, watcher })
    }
}

impl ClipboardSubscribe for Clipboard {
    type Subscriber = Subscriber;

    fn subscribe(&self) -> Result<Self::Subscriber, Error> { self.watcher.subscribe() }
}

impl ClipboardLoad for Clipboard {
    #[inline]
    fn load(&self, mime: &mime::Mime) -> Result<Vec<u8>, Error> { self.getter.load(mime) }
}

impl ClipboardStore for Clipboard {
    #[inline]
    fn store(&self, mime: mime::Mime, data: &[u8]) -> Result<(), Error> {
        self.setter.store(mime, data)
    }

    #[inline]
    fn store_mime_data(&self, mime_data: MimeData) -> Result<(), Error> {
        self.setter.store_mime_data(mime_data)
    }

    #[inline]
    fn clear(&self) -> Result<(), Error> { self.setter.clear() }
}

#[derive(Debug)]
pub struct Context {
    display_name: Option<String>,
    connection: RustConnection,
    window: u32,
    mode: Mode,
    atom_cache: Arc<AtomCache>,
}

impl Context {
    #[inline]
    pub fn new(display_name: Option<&str>, mode: Mode) -> Result<Context, Error> {
        let (connection, window) = Self::new_connection(display_name)?;
        let atom_cache = Arc::new(AtomCache::new(&connection)?);
        let display_name = display_name.map(str::to_owned);
        Ok(Context { display_name, connection, window, atom_cache, mode })
    }

    #[inline]
    pub fn try_clone(&self) -> Result<Context, Error> {
        let display_name = self.display_name.clone();
        let (connection, window) = Self::new_connection(display_name.as_deref())?;

        Ok(Context {
            display_name,
            connection,
            window,
            atom_cache: self.atom_cache.clone(),
            mode: self.mode,
        })
    }

    fn new_connection(
        display_name: Option<&str>,
    ) -> Result<(RustConnection, xproto::Window), Error> {
        let (connection, screen_num) =
            RustConnection::connect(display_name).context(error::Connect)?;

        let window = {
            let window = connection.generate_id().context(error::GenerateX11Identifier)?;
            let screen = &connection.setup().roots[screen_num];

            connection
                .create_window(
                    x11rb::COPY_DEPTH_FROM_PARENT,
                    window,
                    screen.root,
                    0,
                    0,
                    1,
                    1,
                    0,
                    xproto::WindowClass::INPUT_OUTPUT,
                    screen.root_visual,
                    &xproto::CreateWindowAux::default().event_mask(
                        xproto::EventMask::PROPERTY_CHANGE, // | EventMask::STRUCTURE_NOTIFY
                    ),
                )
                .context(error::CreateWindow)?;

            window
        };

        Ok((connection, window))
    }

    #[inline]
    pub fn clipboard_type(&self) -> xproto::Atom {
        match self.mode {
            Mode::Clipboard => self.atom_cache.clipboard_selection,
            Mode::Selection => self.atom_cache.primary_selection,
        }
    }

    #[inline]
    pub fn close_event(&self) -> xproto::ClientMessageEvent {
        const CLOSE_CONNECTION_ATOM: xproto::Atom = 8293;
        xproto::ClientMessageEvent {
            response_type: xproto::CLIENT_MESSAGE_EVENT,
            sequence: 0,
            format: 32,
            window: self.window,
            type_: CLOSE_CONNECTION_ATOM,
            data: [0u32; 5].into(),
        }
    }

    #[inline]
    pub fn is_close_event(&self, event: &xproto::ClientMessageEvent) -> bool {
        let close_event = self.close_event();
        close_event.response_type == event.response_type
            && close_event.format == event.format
            && close_event.sequence == event.sequence
            && close_event.window == event.window
            && close_event.type_ == event.type_
    }

    #[inline]
    pub fn send_close_connection_event(&self) -> Result<(), Error> {
        let close_event = self.close_event();
        self.connection
            .send_event(false, self.window, x11rb::NONE, close_event)
            .context(error::SendEvent)?;
        self.flush_connection()?;
        Ok(())
    }

    #[inline]
    pub fn flush_connection(&self) -> Result<(), Error> {
        self.connection.flush().context(error::FlushConnection)?;
        Ok(())
    }

    #[inline]
    #[allow(unused)]
    pub fn poll_for_event(&self) -> Result<Option<x11rb::protocol::Event>, Error> {
        self.connection.poll_for_event().context(error::PollForEvent)
    }

    #[inline]
    pub fn wait_for_event(&self) -> Result<x11rb::protocol::Event, Error> {
        self.connection.wait_for_event().context(error::WaitForEvent)
    }

    #[inline]
    pub fn get_intern_atom(&self, atom_name: &str) -> Result<xproto::Atom, Error> {
        get_intern_atom(&self.connection, atom_name)
    }

    #[inline]
    pub fn get_intern_atom_from_mime(&self, mime: &mime::Mime) -> Result<xproto::Atom, Error> {
        get_intern_atom_from_mime(&self.connection, mime)
    }
}

#[derive(Debug)]
struct AtomCache {
    clipboard_selection: xproto::Atom,
    primary_selection: xproto::Atom,
    targets: xproto::Atom,
    save_targets: xproto::Atom,
    utf8_string: xproto::Atom,
    incr: xproto::Atom,
}

impl AtomCache {
    fn new(conn: &impl Connection) -> Result<AtomCache, Error> {
        Ok(AtomCache {
            clipboard_selection: get_intern_atom(conn, "CLIPBOARD")?,
            primary_selection: xproto::AtomEnum::PRIMARY.into(),
            targets: get_intern_atom(conn, "TARGETS")?,
            save_targets: get_intern_atom(conn, "SAVE_TARGETS")?,
            utf8_string: get_intern_atom(conn, "UTF8_STRING")?,
            incr: get_intern_atom(conn, "INCR")?,
        })
    }
}

#[inline]
pub fn get_intern_atom(conn: &impl Connection, atom_name: &str) -> Result<xproto::Atom, Error> {
    conn.intern_atom(false, atom_name.as_bytes())
        .with_context(|| error::GetAtomIdentifierByName { atom_name: atom_name.to_string() })?
        .reply()
        .map(|r| r.atom)
        .context(error::Reply)
}

#[inline]
pub fn get_intern_atom_from_mime(
    conn: &impl Connection,
    mime: &mime::Mime,
) -> Result<xproto::Atom, Error> {
    let atom_name = match (mime.type_(), mime.subtype(), mime.get_param(mime::CHARSET)) {
        (mime::TEXT, mime::PLAIN, Some(mime::UTF_8)) => "UTF8_STRING",
        (mime::TEXT, ..) => "UTF8_STRING",
        _ => mime.essence_str(),
    };

    conn.intern_atom(false, atom_name.as_bytes())
        .with_context(|| error::GetAtomIdentifierByName { atom_name: atom_name.to_string() })?
        .reply()
        .map(|r| r.atom)
        .context(error::Reply)
}
