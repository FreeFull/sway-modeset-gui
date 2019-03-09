use std::env::var_os;
use std::fmt;
use std::io::{self, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

use byteorder::{NativeEndian, ReadBytesExt};
use serde::{de::DeserializeOwned, Deserialize};

const MAGIC: &[u8; 6] = b"i3-ipc";

pub type Result<T> = std::result::Result<T, Error>;

pub struct Connection {
    stream: UnixStream,
}

impl Connection {
    pub fn connect() -> Result<Connection> {
        let swaysock = var_os("SWAYSOCK").ok_or(Error::Env)?;
        Connection::connect_with_path(swaysock)
    }

    pub fn connect_with_path<P: AsRef<Path>>(path: P) -> Result<Connection> {
        let stream = UnixStream::connect(path)?;
        stream.set_read_timeout(Some(Duration::from_secs(30)))?;
        stream.set_write_timeout(Some(Duration::from_secs(30)))?;
        Ok(Connection { stream })
    }

    pub fn get_outputs(&mut self) -> Result<Vec<Output>> {
        self.send_message(MessageType::GetOutputs, &"")?;
        let message = self.read_message()?;
        assert_eq!(message.msg_type, MessageType::GetOutputs);
        Ok(message.content)
    }

    fn read_message<T: DeserializeOwned>(&mut self) -> Result<Response<T>> {
        let mut small_buffer = [0; 6];
        self.stream.read_exact(&mut small_buffer[0..6])?;
        if &small_buffer[0..6] != MAGIC {
            return Err(Error::MalformedReply);
        }
        let message_length = self.stream.read_u32::<NativeEndian>()?;
        let message_type = self.stream.read_u32::<NativeEndian>()?;
        let mut buffer = vec![0; message_length as usize];
        self.stream.read_exact(&mut buffer)?;
        Ok(Response {
            msg_type: MessageType::from_u32(message_type)?,
            content: serde_json::from_slice(&buffer)?,
        })
    }

    fn send_message(&mut self, msg_type: MessageType, message: &str) -> Result<()> {
        let msg_type = msg_type as u32;
        self.stream.write(MAGIC)?;
        self.stream.write(&(message.len() as u32).to_ne_bytes())?;
        self.stream.write(&msg_type.to_ne_bytes())?;
        self.stream.write(message.as_bytes())?;
        Ok(())
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MessageType {
    RunCommand,
    GetWorkspaces,
    Subscribe,
    GetOutputs,
    GetTree,
    GetMarks,
    GetBarConfig,
    GetVersion,
    GetBindingModes,
    GetConfig,
    SendTick,
    I3Sync,
}

impl MessageType {
    fn from_u32(msg_type: u32) -> Result<MessageType> {
        Ok(match msg_type {
            0 => MessageType::RunCommand,
            1 => MessageType::GetWorkspaces,
            2 => MessageType::Subscribe,
            3 => MessageType::GetOutputs,
            4 => MessageType::GetTree,
            5 => MessageType::GetMarks,
            6 => MessageType::GetBarConfig,
            7 => MessageType::GetVersion,
            8 => MessageType::GetBindingModes,
            9 => MessageType::GetConfig,
            10 => MessageType::SendTick,
            11 => MessageType::I3Sync,
            _ => return Err(Error::UnknownType),
        })
    }
}

#[derive(Debug)]
pub struct Response<T> {
    pub msg_type: MessageType,
    pub content: T,
}

#[derive(Debug, Deserialize)]
pub struct Output {
    pub active: bool,
    pub id: u32,
    pub name: String,
    pub modes: Vec<Mode>,
    pub make: String,
    pub model: String,
    pub serial: String,
    pub scale: f32,
    pub transform: String,
    pub current_mode: Mode,
}

#[derive(Debug, Deserialize)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Mode {
    pub width: u32,
    pub height: u32,
    pub refresh: u32,
}

#[derive(Debug)]
pub enum Error {
    Env,
    Socket(io::Error),
    Serialisation(serde_json::Error),
    MalformedReply,
    UnknownType,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Env => write!(f, "SWAYSOCK isn't set"),
            Error::Socket(ref err) => write!(f, "{}", err),
            Error::Serialisation(ref err) => write!(f, "{}", err),
            Error::MalformedReply => write!(f, "Malformed reply"),
            Error::UnknownType => write!(f, "Unknown message type received"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Socket(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Serialisation(err)
    }
}
