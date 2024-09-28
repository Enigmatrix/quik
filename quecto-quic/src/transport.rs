pub struct Connection;

impl Connection {
    pub fn send(&self, data: &[u8]) {
        // Send data
    }

    pub fn recv(&self, data: &[u8]) {
        let first_byte = data[0];
        if first_byte & 0b1 == 1 {
            // This is a long header
            if (first_byte >> 1) & 0b1 == 0 {
                // TODO this *must* be a VersionNegotiationPacket
                // check or else return an error
            }
            let packet_type = (first_byte >> 2) & 0b11;
            let version = u32::from_be_bytes(data[1..5]);
            match packet_type {
                0b00 => {
                    // Initial packet
                }
                0b01 => {
                    // 0-RTT packet
                }
                0b10 => {
                    // Handshake packet
                }
                0b11 => {
                    // Retry packet
                }
                _ => unreachable!()
            }
        } else {
            // This is a short header
        }
    }
    
    pub fn close(self) {
        // Close connection
    }
}