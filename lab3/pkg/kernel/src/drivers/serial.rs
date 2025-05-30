use super::uart16550::SerialPort;

const SERIAL_IO_PORT: u16 = 0x3F8; // COM1

once_mutex!(pub SERIAL: SerialPort<SERIAL_IO_PORT>);

pub fn init() {
    init_SERIAL(SerialPort::new());
    get_serial_for_sure().init();

    println!("{}", crate::get_ascii_header());
    println!("[+] Serial Initialized.");
}

guard_access_fn!(pub get_serial(SERIAL: SerialPort<SERIAL_IO_PORT>));

pub fn backspace() {
    let mut serial = get_serial_for_sure();
    serial.send(0x08);
    serial.send(0x20);
    serial.send(0x08);
}
