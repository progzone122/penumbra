mod connection;
mod da;
use connection::{find_mtk_port, get_mtk_port_connection};
use serialport::SerialPortInfo;

fn main() {
    let da_path = std::path::Path::new("DA_penangf.bin");
    let da_data = std::fs::read(da_path).expect("Failed to read DA file");
    let da = da::DAFile::parse_da(&da_data).expect("Failed to parse DA file");


    // let mtk_port: SerialPortInfo;
    // loop {
    //     let ports = find_mtk_port();
    //     if ports.len() > 0 {
    //         mtk_port = ports[0].clone();
    //         break;
    //     }
    //     println!("No MTK ports found. Please connect a device.");

    // }
    // println!("Found MTK port: {}", mtk_port.port_name);
    // if let Some(mut connection) = get_mtk_port_connection(&mtk_port) {
    //     connection.handshake().expect("Handshake failed");
    // } else {
    //     println!("Failed to open MTK connection.");
    // }

    // let connection = Connection {
    //     connection_type: connection::connection::ConnectionType::Brom,
    //     port: String::from("COM3"),
    //     baudrate: 115200,
    // };

    // println!("Connection setup on port: {}", connection.port);
}
