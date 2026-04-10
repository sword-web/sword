use serde::{Deserialize, Serialize};
use socketioxide::{ParserConfig, SocketIo, TransportType, layer::SocketIoLayer};
use socketioxide_parser_common::CommonParser;
use socketioxide_parser_msgpack::MsgPackParser;
use std::collections::HashSet;
use std::str::FromStr;
use sword_core::{ByteConfig, ConfigItem, ConfigRegistrar, TimeConfig, inventory_submit};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SocketIoServerConfig {
    /// Whether to enable the Socket.IO server.
    /// Defaults to false.
    pub enabled: bool,

    /// The amount of time the server will wait for an acknowledgement
    /// from the client before closing the connection.
    ///
    /// Defaults to 5 seconds.
    pub ack_timeout: Option<TimeConfig>,

    /// The amount of time before disconnecting a client that has not
    /// successfully joined a namespace.
    ///
    /// Defaults to 45 seconds.
    pub connect_timeout: Option<TimeConfig>,

    /// The maximum number of packets that can be buffered per connection
    /// before being emitted to the client. If the buffer if full the emit()
    /// method will return an error.
    ///
    /// Defaults to 128 packets.
    pub max_buffer_size: Option<usize>,

    /// The maximum size of a payload in bytes. If a payload is bigger than
    /// this value the emit() method will return an error.
    ///
    /// Defaults to 100 kb.
    pub max_payload: Option<ByteConfig>,

    /// The interval at which the server will send a ping packet to the client.
    /// Defaults to 25 seconds.
    pub ping_interval: Option<TimeConfig>,

    /// The amount of time the server will wait for a ping response from the
    /// client before closing the connection.
    ///
    /// Defaults to 20 seconds.
    pub ping_timeout: Option<TimeConfig>,

    /// The path to listen for socket.io requests on.
    /// Defaults to "/socket.io".
    pub req_path: Option<String>,

    /// The transports to allow for connections.
    /// Valid options are "polling" and "websocket".
    pub transports: Option<Vec<String>>,

    #[serde(default)]
    /// The parser to use for encoding and decoding messages.
    /// Valid options are "common" and "msgpack".
    pub parser: SocketIoParser,

    /// The size of the read buffer for the websocket transport.
    /// You can tweak this value depending on your use case.
    ///
    /// Defaults to 4KiB.
    ///
    /// Setting it to a higher value will improve performance on heavy read scenarios
    /// but will consume more memory.
    pub ws_read_buffer_size: Option<usize>,

    /// Whether to display the configuration on startup.
    pub display: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum SocketIoParser {
    Common(CommonParser),
    MsgPack(MsgPackParser),
}

impl Default for SocketIoParser {
    fn default() -> Self {
        SocketIoParser::Common(CommonParser)
    }
}

impl FromStr for SocketIoParser {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "common" => Ok(SocketIoParser::Common(CommonParser)),
            "msgpack" => Ok(SocketIoParser::MsgPack(MsgPackParser)),
            _ => Err(format!("invalid Socket.IO parser: {s}")),
        }
    }
}

impl std::fmt::Display for SocketIoParser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SocketIoParser::Common(_) => write!(f, "common"),
            SocketIoParser::MsgPack(_) => write!(f, "msgpack"),
        }
    }
}

impl Serialize for SocketIoParser {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for SocketIoParser {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        SocketIoParser::from_str(&s).map_err(serde::de::Error::custom)
    }
}

pub struct SocketIoServerLayer;

impl SocketIoServerLayer {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(config: &SocketIoServerConfig) -> (SocketIoLayer, SocketIo) {
        let mut layer_builder = SocketIo::builder();

        if let Some(ack_timeout) = &config.ack_timeout {
            layer_builder = layer_builder.ack_timeout(ack_timeout.parsed);
        }

        if let Some(connect_timeout) = &config.connect_timeout {
            layer_builder = layer_builder.connect_timeout(connect_timeout.parsed);
        }

        if let Some(max_buffer_size) = &config.max_buffer_size {
            layer_builder = layer_builder.max_buffer_size(*max_buffer_size);
        }

        if let Some(max_payload) = &config.max_payload {
            layer_builder = layer_builder.max_payload(max_payload.parsed as u64);
        }

        if let Some(ping_interval) = &config.ping_interval {
            layer_builder = layer_builder.ping_interval(ping_interval.parsed);
        }

        if let Some(ping_timeout) = &config.ping_timeout {
            layer_builder = layer_builder.ping_timeout(ping_timeout.parsed);
        }

        if let Some(req_path) = &config.req_path {
            layer_builder = layer_builder.req_path(req_path.clone());
        }

        if let Some(transports) = &config.transports {
            let invalid_transports: Vec<&str> = transports
                .iter()
                .map(String::as_str)
                .filter(|t| !matches!(*t, "polling" | "websocket"))
                .collect();

            if !invalid_transports.is_empty() {
                tracing::warn!(
                    target: "sword.socketio.config",
                    invalid_transports = ?invalid_transports,
                    "Ignoring invalid socket.io transports; valid values are 'polling' and 'websocket'"
                );
            }

            let parsed_transports = transports
                .iter()
                .collect::<HashSet<_>>()
                .iter()
                .filter_map(|t| match t.as_str() {
                    "polling" => Some(TransportType::Polling),
                    "websocket" => Some(TransportType::Websocket),
                    _ => None,
                })
                .collect::<Vec<_>>();

            match parsed_transports.len() {
                1 => layer_builder = layer_builder.transports([parsed_transports[0]]),
                2 => {
                    layer_builder =
                        layer_builder.transports([parsed_transports[0], parsed_transports[1]])
                }
                _ => {}
            };
        }

        let parser_config = match config.parser {
            SocketIoParser::Common(_) => ParserConfig::common(),
            SocketIoParser::MsgPack(_) => ParserConfig::msgpack(),
        };

        layer_builder = layer_builder.with_parser(parser_config);

        if let Some(ws_read_buffer_size) = config.ws_read_buffer_size {
            layer_builder = layer_builder.ws_read_buffer_size(ws_read_buffer_size);
        }

        let (layer, io) = layer_builder.build_layer();

        (layer, io)
    }
}

impl ConfigItem for SocketIoServerConfig {
    fn key() -> &'static str {
        "socketio"
    }
}

inventory_submit! {[
    ConfigRegistrar::new(|state, config| {
        state.insert(config.get_or_default::<SocketIoServerConfig>());
    })
]}
