use core::fmt::{self, Formatter};

use crate::result::{Error, Result, ResultCode};
use crate::PacketBuffer;

#[derive(Debug)]
pub struct Header {
    /// A random identifier is assigned to query packets. Response packets must reply with the
    /// same id. This is needed to differentiate responses due to the stateless nature of UDP.
    id: u16,

    /// 1 bit. 0 for queries, 1 for responses.
    is_response: bool,
    /// 4 bits. Typically always 0, see RFC1035 for details.
    _op_code: u8,
    /// 1 bit. Set to 1 if the responding server is authoritative - that is, it "owns" - the domain queried.
    is_authoritative: bool,
    /// 1 bit. Set to 1 if the message length exceeds 512 bytes. Traditionally a hint that the
    /// query can be reissued using TCP, for which the length limitation doesn't apply.
    is_truncated: bool,
    /// 1 bit. Set by the sender of the request if the server should attempt to resolve the query recursively if it does not have an answer readily available.
    recursion_desired: bool,
    /// 1 bit. Set by the server to indicate whether or not recursive queries are allowed.
    recursion_available: bool,
    /// 3 bits. Originally reserved for later use, but now used for DNSSEC queries.
    _z: u8,

    /// 4 bits. Set by the server to indicate the status of the response, i.e. whether or not it was successful or failed, and in the latter case providing details about the cause of the failure.
    response_code: ResultCode,

    /// 16 bits. The number of entries in the Question Section.
    pub question_count: u16,
    /// 16 bits. The number of entries in the Answer Section.
    pub answer_count: u16,
    /// 16 bits. The number of entries in the Authority Section.
    pub authority_count: u16,
    /// 16 bits. The number of entries in the Additional Section.
    pub additional_count: u16,
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Header {{")?;
        writeln!(f, "\tID: {}", self.id)?;
        writeln!(
            f,
            "\tis_response: {}",
            if self.is_response { "1" } else { "0" }
        )?;
        writeln!(
            f,
            "\tis_authoritative: {}",
            if self.is_authoritative { "1" } else { "0" }
        )?;
        writeln!(
            f,
            "\tis_truncated: {}",
            if self.is_truncated { "1" } else { "0" }
        )?;
        writeln!(
            f,
            "\tRec. Desired: {}",
            if self.recursion_desired { "1" } else { "0" }
        )?;
        writeln!(
            f,
            "\tRec. Available: {}",
            if self.recursion_available { "1" } else { "0" }
        )?;
        writeln!(f, "\tRCODE: {}", self.response_code)?;
        writeln!(f, "\tNB Questions: {}", self.question_count)?;
        writeln!(f, "\tNB Answers: {}", self.answer_count)?;
        writeln!(f, "\tNB Authorities: {}", self.authority_count)?;
        writeln!(f, "\tNB Additionals: {}", self.additional_count)?;

        writeln!(f, "}}")?;

        Ok(())
    }
}

impl TryFrom<&mut PacketBuffer> for Header {
    type Error = Error;

    fn try_from(buffer: &mut PacketBuffer) -> Result<Self> {
        if buffer.pos() != 0 {
            return Err(Error::PacketBufferInvalidPosition); //("Packet buffer must be at position 0 before reading Header.");
        }

        // println!("RAW HEADER: {:#?}", &buffer.bytes[0..40]);
        let id = buffer.read_u16()?;

        // First 8 bits
        let byte = buffer.read_u8()?;
        let is_response = byte & 0x80 != 0;
        let _op_code = (byte & 0x74) >> 3;
        let is_authoritative = (byte & 0x04) != 0;
        let is_truncated = (byte & 0x02) != 0;
        let recursion_desired = (byte & 0x01) != 0;

        // Next 8 bits
        let byte = buffer.read_u8()?;
        let recursion_available = (byte & 0x80) != 0;
        let _z = (byte & 0x70) >> 5;
        let response_code = ResultCode::from(byte & 0x0f);

        let question_count = buffer.read_u16()?;
        let answer_count = buffer.read_u16()?;
        let authority_count = buffer.read_u16()?;
        let additional_count = buffer.read_u16()?;

        Ok(Self {
            id,

            is_response,
            _op_code,
            is_authoritative,
            is_truncated,
            recursion_desired,

            recursion_available,
            _z,
            response_code,

            question_count,
            answer_count,
            authority_count,
            additional_count,
        })
    }
}
