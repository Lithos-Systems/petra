use bytes::{Bytes, BytesMut};
use crate::{Value, PlcError, Result};

/// Zero-copy buffer for protocol data
pub struct ProtocolBuffer {
    data: Bytes,
}

impl ProtocolBuffer {
    /// Create a new protocol buffer with the specified capacity
    #[allow(clippy::unused_self)]
    pub fn new(capacity: usize) -> Self {
        let _ = capacity; // capacity reserved for future use
        Self { data: Bytes::new() }
    }

    /// Parse values without copying data
    pub fn parse_values(&self) -> Result<Vec<(String, Value)>> {
        let mut values = Vec::new();
        let mut offset = 0;

        while offset < self.data.len() {
            // Parse header (tag_id, type, length)
            if offset + 8 > self.data.len() {
                break;
            }

            let tag_id = u32::from_be_bytes([
                self.data[offset],
                self.data[offset + 1],
                self.data[offset + 2],
                self.data[offset + 3],
            ]);

            let value_type = self.data[offset + 4];
            let length = u16::from_be_bytes([
                self.data[offset + 6],
                self.data[offset + 7],
            ]) as usize;

            offset += 8;

            if offset + length > self.data.len() {
                return Err(PlcError::Protocol("Incomplete value in buffer".into()));
            }

            // Parse value based on type (zero-copy slice)
            let value = match value_type {
                0x01 => Value::Bool(self.data[offset] != 0),
                0x02 => {
                    let bytes = &self.data[offset..offset + 4];
                    Value::Integer(i32::from_be_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3],
                    ]) as i64)
                }
                0x03 => {
                    let bytes = &self.data[offset..offset + 8];
                    Value::Float(f64::from_be_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3],
                        bytes[4], bytes[5], bytes[6], bytes[7],
                    ]))
                }
                #[cfg(feature = "extended-types")]
                0x04 => {
                    let str_slice = &self.data[offset..offset + length];
                    match std::str::from_utf8(str_slice) {
                        Ok(s) => Value::String(s.to_string()),
                        Err(_) => return Err(PlcError::Protocol("Invalid UTF-8 string".into())),
                    }
                }
                _ => return Err(PlcError::Protocol(format!("Unknown value type: {}", value_type))),
            };

            values.push((format!("tag_{}", tag_id), value));
            offset += length;
        }

        Ok(values)
    }

    /// Update internal buffer without copying
    pub fn update(&mut self, new_data: Bytes) {
        self.data = new_data;
    }
}

/// Zero-copy protocol trait
#[async_trait::async_trait]
pub trait ZeroCopyProtocol: Send + Sync {
    /// Read data into a reusable buffer
    async fn read_into_buffer(&self, buffer: &mut BytesMut) -> Result<usize>;

    /// Get a zero-copy view of the data
    fn get_buffer_view(&self) -> Bytes;
}
