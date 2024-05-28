use nom_derive::{Nom, Parse};

use crate::static_versions::{v5::V5, v7::V7};
use crate::variable_versions::ipfix::IPFix;
use crate::variable_versions::v9::V9;
use crate::{NetflowPacketResult, NetflowParser};

/// Struct is used simply to match how to handle the result of the packet
#[derive(Nom)]
pub struct NetflowHeader {
    /// Netflow Version
    pub version: u16,
}

pub enum NetflowHeaderResult<'a> {
    V5(&'a [u8]),
    V7(&'a [u8]),
    V9(&'a [u8]),
    IPFix(&'a [u8]),
}

impl NetflowHeader {
    pub fn parse_header(
        packet: &[u8],
    ) -> Result<NetflowHeaderResult, Box<dyn std::error::Error>> {
        match NetflowHeader::parse_be(packet) {
            Ok((i, header)) if header.version == 5 => Ok(NetflowHeaderResult::V5(i)),
            Ok((i, header)) if header.version == 7 => Ok(NetflowHeaderResult::V7(i)),
            Ok((i, header)) if header.version == 9 => Ok(NetflowHeaderResult::V9(i)),
            Ok((i, header)) if header.version == 10 => Ok(NetflowHeaderResult::IPFix(i)),
            _ => Err(("Unsupported Version").into()),
        }
    }
}

pub type V5ParsedResult<'a> = (&'a [u8], V5);
pub type V7ParsedResult<'a> = (&'a [u8], V7);
pub type V9ParsedResult<'a> = (&'a [u8], V9);
pub type IPFixParsedResult<'a> = (&'a [u8], IPFix);

impl<'a> From<V5ParsedResult<'a>> for ParsedNetflow {
    fn from((remaining, v5_parsed): V5ParsedResult) -> ParsedNetflow {
        Self::new(remaining, NetflowPacketResult::V5(v5_parsed))
    }
}

impl<'a> From<V7ParsedResult<'a>> for ParsedNetflow {
    fn from((remaining, v7_parsed): V7ParsedResult) -> ParsedNetflow {
        Self::new(remaining, NetflowPacketResult::V7(v7_parsed))
    }
}

impl<'a> From<V9ParsedResult<'a>> for ParsedNetflow {
    fn from((remaining, v9_parsed): V9ParsedResult) -> ParsedNetflow {
        Self::new(remaining, NetflowPacketResult::V9(v9_parsed))
    }
}

impl<'a> From<IPFixParsedResult<'a>> for ParsedNetflow {
    fn from((remaining, ipfix_parsed): IPFixParsedResult) -> ParsedNetflow {
        Self::new(remaining, NetflowPacketResult::IPFix(ipfix_parsed))
    }
}

#[derive(Debug, Clone)]
pub struct ParsedNetflow {
    pub remaining: Vec<u8>,
    /// Parsed Netflow Packet
    pub netflow_packet: NetflowPacketResult,
}

impl ParsedNetflow {
    fn new(remaining: &[u8], netflow_packet: NetflowPacketResult) -> Self {
        Self {
            remaining: remaining.to_vec(),
            netflow_packet,
        }
    }
}

pub trait Parser {
    fn parse<'a>(
        &'a mut self,
        packet: &'a [u8],
    ) -> Result<ParsedNetflow, Box<dyn std::error::Error>>;
}

impl Parser for NetflowParser {
    /// Parses a Netflow by version packet and returns a Parsed Netflow.
    fn parse<'a>(
        &'a mut self,
        packet: &'a [u8],
    ) -> Result<ParsedNetflow, Box<dyn std::error::Error>> {
        match NetflowHeader::parse_header(packet) {
            Ok(NetflowHeaderResult::V5(v5_packet)) => V5::parse(v5_packet)
                .map(|r: V5ParsedResult| r.into())
                .map_err(|e| format!("Could not parse V5 packet: {e}").into()),
            Ok(NetflowHeaderResult::V7(v7_packet)) => V7::parse(v7_packet)
                .map(|r: V7ParsedResult| r.into())
                .map_err(|e| format!("Could not parse V7 packet: {e}").into()),
            Ok(NetflowHeaderResult::V9(v9_packet)) => V9::parse(v9_packet, &mut self.v9_parser)
                .map(|r: V9ParsedResult| r.into())
                .map_err(|e| format!("Could not parse V9 packet: {e}").into()),
            Ok(NetflowHeaderResult::IPFix(ipfix_packet)) => {
                IPFix::parse(ipfix_packet, &mut self.ipfix_parser)
                    .map(|r: IPFixParsedResult| r.into())
                    .map_err(|e| format!("Could not parse v10 packet: {e}").into())
            }
            Err(e) => Err(e),
        }
    }
}
