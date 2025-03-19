use core::fmt;
use x86_64::instructions::port::Port;

/// A port-mapped UART 16550 serial interface.
pub struct SerialPort<const BASE_ADDR: u16> {
    data: Port<u8>,
    int_enable: Port<u8>,
    fifo_control: Port<u8>,
    line_control: Port<u8>,
    modem_control: Port<u8>,
    line_status: Port<u8>,
}

impl<const BASE_ADDR: u16> SerialPort<BASE_ADDR> {
    pub const fn new() -> Self {
        Self {
            data: Port::new(BASE_ADDR),
            int_enable: Port::new(BASE_ADDR + 1),
            fifo_control: Port::new(BASE_ADDR + 2),
            line_control: Port::new(BASE_ADDR + 3),
            modem_control: Port::new(BASE_ADDR + 4),
            line_status: Port::new(BASE_ADDR + 5),
        }
    }

    /// Initializes the serial port.
    pub fn init(&mut self) {
        // FIXME: Initialize the serial port
        unsafe {
            self.int_enable.write(0x00_u8); // Disable all interrupts
            self.line_control.write(0x80_u8); // Enable DLAB (set baud rate divisor)
            self.data.write(0x03_u8); // Set divisor to 3 (lo byte) 38400 baud
            self.int_enable.write(0x00_u8); //                  (hi byte)
            self.line_control.write(0x03_u8); // 8 bits, no parity, one stop bit
            self.fifo_control.write(0xC7_u8); // Enable FIFO, clear them, with 14-byte threshold
            self.modem_control.write(0x0B_u8); // IRQs enabled, RTS/DSR set
            self.modem_control.write(0x1E_u8); // Set in loopback mode, test the serial chip
            self.data.write(0xAE_u8); // Test serial chip (send byte 0xAE and check if serial returns same byte)
            if self.data.read() != 0xAE_u8 {
                panic!("Serial port initialization failed.");
            }
            self.modem_control.write(0x0F_u8);
        }
    }

    /// Sends a byte on the serial port.
    pub fn send(&mut self, data: u8) {
        // FIXME: Send a byte on the serial port
        unsafe {
            while self.line_status.read() & 0x20 == 0 {}
            self.data.write(data);
        }
    }
    pub fn receive(&mut self) -> Option<u8> {
        // FIXME: Receive a byte on the serial port no wait
        unsafe {
            if self.line_status.read() & 1 == 0 {
                None
            } else {
                Some(self.data.read())
            }
        }
    }
}

impl<const BASE_ADDR: u16> fmt::Write for SerialPort<BASE_ADDR> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.send(byte);
        }
        Ok(())
    }
}
