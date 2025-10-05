//! Variable related commands (`ReadVar`, `WriteVar`)

use super::command_trait::Command;
use crate::{HsesPayload, error::ProtocolError};
use std::marker::PhantomData;

/// Command ID mapping for variable types
pub trait VariableCommandId {
    fn command_id() -> u16;
}

// Implement VariableCommandId for each variable type
impl VariableCommandId for u8 {
    fn command_id() -> u16 {
        0x7a
    }
}

impl VariableCommandId for i16 {
    fn command_id() -> u16 {
        0x7b
    }
}

impl VariableCommandId for i32 {
    fn command_id() -> u16 {
        0x7c
    }
}

impl VariableCommandId for f32 {
    fn command_id() -> u16 {
        0x7d
    }
}

impl VariableCommandId for Vec<u8> {
    fn command_id() -> u16 {
        0x7e
    }
}

pub struct ReadVar<T: HsesPayload + VariableCommandId> {
    pub index: u8,
    pub _phantom: PhantomData<T>,
}

impl<T: HsesPayload + VariableCommandId> Command for ReadVar<T> {
    type Response = T;
    fn command_id() -> u16 {
        T::command_id()
    }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        Ok(Vec::new())
    }
    fn instance(&self) -> u16 {
        u16::from(self.index) // Variable number (0-99 for byte, 0-999 for int/real)
    }
    fn attribute(&self) -> u8 {
        1 // Fixed to 1 according to specification
    }
    fn service(&self) -> u8 {
        0x0e // Get_Attribute_Single
    }
}

pub struct WriteVar<T: HsesPayload + VariableCommandId> {
    pub index: u8,
    pub value: T,
}

impl<T: HsesPayload + VariableCommandId> Command for WriteVar<T> {
    type Response = ();
    fn command_id() -> u16 {
        T::command_id()
    }
    fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        // WriteVar requires encoding, but Command trait doesn't support it
        // This is a design limitation - we'll use UTF-8 as default
        self.value.serialize(crate::encoding::TextEncoding::Utf8)
    }
    fn instance(&self) -> u16 {
        u16::from(self.index) // Variable number (0-99 for byte, 0-999 for int/real)
    }
    fn attribute(&self) -> u8 {
        1 // Fixed to 1 according to specification
    }
    fn service(&self) -> u8 {
        0x10 // Set_Attribute_Single
    }
}
