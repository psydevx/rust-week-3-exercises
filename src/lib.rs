use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::Deref;


#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct CompactSize {
    pub value: u64,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BitcoinError {
    InsufficientBytes,
    InvalidFormat,
}

impl CompactSize {
    pub fn new(value: u64) -> Self {
        // TODO: Construct a CompactSize from a u64 value
        CompactSize { value }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Encode according to Bitcoin's CompactSize format:
        // [0x00–0xFC] => 1 byte
        // [0xFDxxxx] => 0xFD + u16 (2 bytes)
        // [0xFExxxxxxxx] => 0xFE + u32 (4 bytes)
        // [0xFFxxxxxxxxxxxxxxxx] => 0xFF + u64 (8 bytes)
        match self.value {
            0..=252 => vec![self.value as u8],
            253..=65535 => {
                let mut result = vec![0xFD];
                result.extend_from_slice(&(self.value as u16).to_le_bytes());
                result
            }
            65536..=4294967295 => {
                let mut result = vec![0xFE];
                result.extend_from_slice(&(self.value as u32).to_le_bytes());
                result
            }
            _ => {
                let mut result = vec![0xFF];
                result.extend_from_slice(&self.value.to_le_bytes());
                result
            }
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        // TODO: Decode CompactSize, returning value and number of bytes consumed.
        // First check if bytes is empty.
        // Check that enough bytes are available based on prefix.
        if bytes.is_empty() {
            return Err(BitcoinError::InsufficientBytes);
        }

        match bytes[0] {
            0..=252 => Ok((CompactSize { value: bytes[0] as u64 }, 1)),
            0xFD => {
                if bytes.len() < 3 {
                    return Err(BitcoinError::InsufficientBytes);
                }
                let value = u16::from_le_bytes([bytes[1], bytes[2]]) as u64;
                Ok((CompactSize { value }, 3))
            }
            0xFE => {
                if bytes.len() < 5 {
                    return Err(BitcoinError::InsufficientBytes);
                }
                let value = u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as u64;
                Ok((CompactSize { value }, 5))
            }
            0xFF => {
                if bytes.len() < 9 {
                    return Err(BitcoinError::InsufficientBytes);
                }
                let value = u64::from_le_bytes([
                    bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8],
                ]);
                Ok((CompactSize { value }, 9))
            }
            _ => Err(BitcoinError::InvalidFormat),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Txid(pub [u8; 32]);

impl Serialize for Txid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // TODO: Serialize as a hex-encoded string (32 bytes => 64 hex characters)
        serializer.serialize_str(&hex::encode(&self.0))
    }
}

impl<'de> Deserialize<'de> for Txid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // TODO: Parse hex string into 32-byte array
        // Use `hex::decode`, validate length = 32
        let hex_str = String::deserialize(deserializer)?;
        let bytes = hex::decode(&hex_str).map_err(serde::de::Error::custom)?;
        if bytes.len() != 32 {
            return Err(serde::de::Error::custom("txid must be 32 bytes"));
        }
        let mut txid = [0u8; 32];
        txid.copy_from_slice(&bytes);
        Ok(Txid(txid))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct OutPoint {
    pub txid: Txid,
    pub vout: u32,
}

impl OutPoint {
    pub fn new(txid: [u8; 32], vout: u32) -> Self {
        // TODO: Create an OutPoint from raw txid bytes and output index
        OutPoint {
            txid: Txid(txid),
            vout,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Serialize as: txid (32 bytes) + vout (4 bytes, little-endian)
        let mut result = Vec::new();
        result.extend_from_slice(&self.txid.0);
        result.extend_from_slice(&self.vout.to_le_bytes());
        result
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        // TODO: Deserialize 36 bytes: txid[0..32], vout[32..36]
        // Return error if insufficient bytes
        if bytes.len() < 36 {
            return Err(BitcoinError::InsufficientBytes);
        }

        let mut txid = [0u8; 32];
        txid.copy_from_slice(&bytes[0..32]);
        let vout = u32::from_le_bytes([bytes[32], bytes[33], bytes[34], bytes[35]]);

        Ok((OutPoint::new(txid, vout), 36))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Script {
    pub bytes: Vec<u8>,
}

impl Script {
    pub fn new(bytes: Vec<u8>) -> Self {
        // TODO: Simple constructor
        Script { bytes }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Prefix with CompactSize (length), then raw bytes
        let mut result = CompactSize::new(self.bytes.len() as u64).to_bytes();
        result.extend_from_slice(&self.bytes);
        result
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        // TODO: Parse CompactSize prefix, then read that many bytes
        // Return error if not enough bytes
        let (size, size_len) = CompactSize::from_bytes(bytes)?;
        let script_len = size.value as usize;
        
        if bytes.len() < size_len + script_len {
            return Err(BitcoinError::InsufficientBytes);
        }

        let script_bytes = bytes[size_len..size_len + script_len].to_vec();
        Ok((Script::new(script_bytes), size_len + script_len))
    }
}

impl Deref for Script {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        // TODO: Allow &Script to be used as &[u8]
        &self.bytes
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct TransactionInput {
    pub previous_output: OutPoint,
    pub script_sig: Script,
    pub sequence: u32,
}

impl TransactionInput {
    pub fn new(previous_output: OutPoint, script_sig: Script, sequence: u32) -> Self {
        // TODO: Basic constructor
        TransactionInput {
            previous_output,
            script_sig,
            sequence,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Serialize: OutPoint + Script (with CompactSize) + sequence (4 bytes LE)
        let mut result = self.previous_output.to_bytes();
        result.extend_from_slice(&self.script_sig.to_bytes());
        result.extend_from_slice(&self.sequence.to_le_bytes());
        result
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        // TODO: Deserialize in order:
        // - OutPoint (36 bytes)
        // - Script (with CompactSize)
        // - Sequence (4 bytes)
        let (outpoint, mut offset) = OutPoint::from_bytes(bytes)?;
        let (script, script_len) = Script::from_bytes(&bytes[offset..])?;
        offset += script_len;

        if bytes.len() < offset + 4 {
            return Err(BitcoinError::InsufficientBytes);
        }

        let sequence = u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        offset += 4;

        Ok((
            TransactionInput::new(outpoint, script, sequence),
            offset,
        ))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct BitcoinTransaction {
    pub version: u32,
    pub inputs: Vec<TransactionInput>,
    pub lock_time: u32,
}

impl BitcoinTransaction {
    pub fn new(version: u32, inputs: Vec<TransactionInput>, lock_time: u32) -> Self {
        // TODO: Construct a transaction from parts
        BitcoinTransaction {
            version,
            inputs,
            lock_time,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Format:
        // - version (4 bytes LE)
        // - CompactSize (number of inputs)
        // - each input serialized
        // - lock_time (4 bytes LE)
        let mut result = Vec::new();
        result.extend_from_slice(&self.version.to_le_bytes());
        result.extend_from_slice(&CompactSize::new(self.inputs.len() as u64).to_bytes());
        for input in &self.inputs {
            result.extend_from_slice(&input.to_bytes());
        }
        result.extend_from_slice(&self.lock_time.to_le_bytes());
        result
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        // TODO: Read version, CompactSize for input count
        // Parse inputs one by one
        // Read final 4 bytes for lock_time
        if bytes.len() < 4 {
            return Err(BitcoinError::InsufficientBytes);
        }

        let version = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let mut offset = 4;

        let (input_count, count_len) = CompactSize::from_bytes(&bytes[offset..])?;
        offset += count_len;

        let mut inputs = Vec::new();
        for _ in 0..input_count.value {
            let (input, input_len) = TransactionInput::from_bytes(&bytes[offset..])?;
            inputs.push(input);
            offset += input_len;
        }

        if bytes.len() < offset + 4 {
            return Err(BitcoinError::InsufficientBytes);
        }

        let lock_time = u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        offset += 4;

        Ok((
            BitcoinTransaction::new(version, inputs, lock_time),
            offset,
        ))
    }
}

impl fmt::Display for BitcoinTransaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: Format a user-friendly string showing version, inputs, lock_time
        // Display scriptSig length and bytes, and previous output info
        write!(f, "Version: {}\n", self.version)?;
        write!(f, "Inputs: {}\n", self.inputs.len())?;
        for (i, input) in self.inputs.iter().enumerate() {
            write!(f, "  Input {}: ", i)?;
            write!(
                f,
                "Previous Output Vout: {}, ScriptSig Length: {}\n",
                input.previous_output.vout,
                input.script_sig.bytes.len()
            )?;
        }
        write!(f, "Lock Time: {}", self.lock_time)?;
        Ok(())
    }
}
