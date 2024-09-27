pub struct Connection;

impl Connection {
    pub fn send(&self, data: &[u8]) {
        // Send data
    }

    pub fn recv(&self, data: &[u8]) {
        // Recv data
    }
    
    pub fn close(self) {
        // Close connection
    }
}